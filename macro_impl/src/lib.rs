use std::collections::HashSet;

use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, Data, DeriveInput, Field, Fields, Ident,
    Meta, NestedMeta, Type, Visibility,
};

struct GlobalAttributes {
    new_struct_name: Option<String>,
    extra_derive: Vec<String>,
    field_attributes: GlobalFieldAttributes,
}

struct GlobalFieldAttributes {
    default_wrapping_behavior: bool,
    make_fields_public: bool,
}

impl GlobalAttributes {
    // TODO: should use named arguments
    fn new(attr: &AttributeArgs) -> Self {
        let new_struct_name = attr.get(0).map(GlobalAttributes::get_new_name);
        let default_wrapping_behavior = attr
            .get(1)
            .map(GlobalAttributes::get_wrapping)
            .unwrap_or(true);
        GlobalAttributes {
            new_struct_name,
            extra_derive: vec!["Clone", "PartialEq", "Default", "Debug"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect(),
            field_attributes: GlobalFieldAttributes {
                default_wrapping_behavior,
                // TODO;
                make_fields_public: true,
            },
        }
    }

    fn get_new_name(ns: &NestedMeta) -> String {
        let m = if let NestedMeta::Meta(m) = ns {
            m
        } else {
            panic!("Only NestedMeta are accepted");
        };
        let p = match m {
            Meta::Path(p) => p,
            Meta::NameValue(_) | Meta::List(_) => {
                panic!("Expecting a path for first argument of 'optional_struct'")
            }
        };
        p.segments
            .last()
            .expect("How can we have an empty path here?")
            .ident
            .to_string()
    }

    fn get_wrapping(ns: &NestedMeta) -> bool {
        let lit = if let NestedMeta::Lit(lit) = ns {
            lit
        } else {
            panic!("Only literal booleans are accepted for 2nd argument of 'optional_struct'");
        };
        match lit {
            syn::Lit::Bool(lb) => lb.value,
            _ => panic!("Only literal booleans are accepted for 2nd argument of 'optional_struct'"),
        }
    }
}

fn set_new_struct_name(new_name: Option<String>, new_struct: &mut DeriveInput) {
    let new_struct_name =
        new_name.unwrap_or_else(|| "Optional".to_owned() + &new_struct.ident.to_string());

    new_struct.ident = Ident::new(&new_struct_name, new_struct.ident.span());
}

fn iter_struct_fields(the_struct: &mut DeriveInput, global_att: Option<&GlobalFieldAttributes>) {
    // TODO: has to be a cleaner way
    let (apply_attribute_metadata, default_wrapping, make_fields_public) = match global_att {
        Some(ga) => (true, ga.default_wrapping_behavior, ga.make_fields_public),
        None => (false, false, false),
    };
    let data_struct = match &mut the_struct.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("OptionalStruct only works for structs :)"),
    };

    let fields = match &mut data_struct.fields {
        Fields::Unnamed(f) => &mut f.unnamed,
        Fields::Named(f) => &mut f.named,
        Fields::Unit => unreachable!("A struct cannot have simply a unit field?"),
    };

    for field in fields.iter_mut() {
        let field_meta_data = extract_relevant_attributes(field, default_wrapping);
        if apply_attribute_metadata {
            field_meta_data.apply_to_field(field);
            if make_fields_public {
                field.vis = Visibility::Public(syn::VisPublic {
                    pub_token: syn::Token![pub](field.vis.span()),
                })
            }
        }
    }
}

fn set_new_struct_fields(new_struct: &mut DeriveInput, global_att: &GlobalFieldAttributes) {
    iter_struct_fields(new_struct, global_att.into())
}

fn remove_optional_struct_attributes(original_struct: &mut DeriveInput) {
    // Last boolean isn't actually used but w/e
    iter_struct_fields(original_struct, None)
}

struct FieldAttributeData {
    wrap: bool,
    new_type: Option<proc_macro2::TokenTree>,
}

impl FieldAttributeData {
    fn apply_to_field(self, f: &mut Field) {
        let mut new_type = if let Some(t) = self.new_type {
            quote! {#t}
        } else {
            let t = &f.ty;
            quote! {#t}
        };

        if self.wrap {
            new_type = quote! {Option<#new_type>};
        };
        f.ty = Type::Verbatim(new_type);
    }
}

fn extract_relevant_attributes(field: &mut Field, default_wrapping: bool) -> FieldAttributeData {
    const RENAME_ATTRIBUTE: &str = "optional_rename";
    const SKIP_WRAP_ATTRIBUTE: &str = "optional_skip_wrap";
    const WRAP_ATTRIBUTE: &str = "optional_wrap";

    let mut field_attribute_data = FieldAttributeData {
        wrap: default_wrapping,
        new_type: None,
    };
    let indexes_to_remove = field
        .attrs
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            if a.path.is_ident(RENAME_ATTRIBUTE) {
                let args = a
                    .parse_args()
                    .expect("'{RENAME_ATTRIBUTE}' attribute expects one and only one argument (the new type to use)");
                field_attribute_data.new_type = Some(args);
                Some(i)
            }
            else if a.path.is_ident(SKIP_WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = false;
                Some(i)
            }
            else if a.path.is_ident(WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = true;
                Some(i)
            }
            else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Don't forget to reverse so the indices are removed without being shifted!
    for i in indexes_to_remove.into_iter().rev() {
        field.attrs.swap_remove(i);
    }
    field_attribute_data
}

fn acc_assigning<T: std::iter::Iterator<Item = U>, U: std::borrow::Borrow<V>, V: ToTokens>(
    idents: T,
) -> proc_macro2::TokenStream {
    let mut acc = quote! {};
    for ident in idents {
        let ident = ident.borrow();
        acc = quote! {
            #acc
            self.#ident.apply_to(&mut t.#ident);
        };
    }
    acc
}

fn generate_apply_fn(
    derive_input: &DeriveInput,
    new_struct: &DeriveInput,
) -> proc_macro2::TokenStream {
    let orig_name = &derive_input.ident;
    let new_name = &new_struct.ident;

    let fields = match &derive_input.data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let acc = match &fields {
        Fields::Unit => unreachable!(),
        Fields::Named(fields_named) => {
            let it = fields_named.named.iter().map(|f| f.ident.as_ref().unwrap());
            acc_assigning::<_, _, Ident>(it)
        }
        Fields::Unnamed(fields_unnamed) => {
            let it = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let i = syn::Index::from(i);
                quote! {#i}
            });
            acc_assigning(it)
        }
    };

    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();
    quote! {
        impl #impl_generics Applyable<#orig_name #ty_generics> #where_clause for Option<#new_name #ty_generics >{
            fn apply_to(self, t: &mut #orig_name #ty_generics) {
                if let Some(s) = self {
                    s.apply_to(t);
                }
            }
        }

        impl #impl_generics Applyable<#orig_name #ty_generics> #where_clause for #new_name #ty_generics {
            fn apply_to(self, t: &mut #orig_name #ty_generics) {
                #acc
            }
        }
    }
}

fn get_derive_macros(
    new_struct: &mut DeriveInput,
    extra_derive: &[String],
) -> proc_macro2::TokenStream {
    let mut extra_derive = extra_derive.iter().collect::<HashSet<_>>();
    for att in &mut new_struct.attrs {
        let ml = if let Ok(Meta::List(ml)) = att.parse_meta() {
            ml
        } else {
            continue;
        };

        if !ml.path.is_ident("derive") {
            continue;
        }

        for n in ml.nested {
            let trait_name = if let NestedMeta::Meta(Meta::Path(m)) = n {
                m
            } else {
                continue;
            };
            // TODO: this *will* panic
            let full_path = quote! { #trait_name };
            extra_derive.remove(&full_path.to_string());
        }
    }

    let mut acc = quote! {};
    for left_trait_to_derive in extra_derive {
        let left_trait_to_derive = format_ident!("{left_trait_to_derive}");
        acc = quote! { #left_trait_to_derive, #acc};
    }

    quote! { #[derive(#acc)] }
}

#[proc_macro_attribute]
pub fn optional_struct(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let global_att = GlobalAttributes::new(&attr);
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    let mut new_struct = derive_input.clone();

    set_new_struct_name(global_att.new_struct_name, &mut new_struct);
    set_new_struct_fields(&mut new_struct, &global_att.field_attributes);
    let derives = get_derive_macros(&mut new_struct, &global_att.extra_derive);
    // https://github.com/rust-lang/rust/issues/65823 :(
    remove_optional_struct_attributes(&mut derive_input);
    let apply_fn_impl = generate_apply_fn(&derive_input, &new_struct);

    let output = quote! {
        #derive_input

        #derives
        #new_struct

        #apply_fn_impl
    };
    proc_macro::TokenStream::from(output)
}
