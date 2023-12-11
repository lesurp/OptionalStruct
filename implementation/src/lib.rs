use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Field, Fields, Ident, Path, spanned::Spanned, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream};

#[cfg(test)]
mod test;

struct StructAttribute {
    new_struct_name: Option<String>,
    default_wrapping: bool,
}

impl Parse for StructAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = StructAttribute {
            new_struct_name: None,
            default_wrapping: true,
        };

        if let Ok(struct_name) = Ident::parse(input) {
            out.new_struct_name = Some(struct_name.to_string());
        } else {
            return Ok(out);
        };

        if input.parse::<Token![,]>().is_err() {
            return Ok(out);
        };

        if let Ok(wrapping) = syn::LitBool::parse(input) {
            out.default_wrapping = wrapping.value;
        } else {
            return Ok(out);
        };

        Ok(out)
    }
}

// TODO this breaks for e.g. yolo::my::Option
fn is_path_option(p: &Path) -> bool {
    p.segments
        .last()
        .map(|ps| ps.ident == "Option")
        .unwrap_or(false)
}

fn is_type_option(t: &Type) -> bool {
    macro_rules! wtf {
        ($reason : tt) => {
            panic!(
                "Using OptionalStruct for a struct containing a {} is dubious...",
                $reason
            )
        };
    }

    match &t {
        // real work
        Type::Path(type_path) => is_path_option(&type_path.path),
        Type::Array(_) | Type::Tuple(_) => false,
        Type::Paren(type_paren) => is_type_option(&type_paren.elem),

        // No clue what to do with those
        Type::ImplTrait(_) | Type::TraitObject(_) => {
            panic!("Might already be an option I have no way to tell :/")
        }
        Type::Infer(_) => panic!("If you cannot tell, neither can I"),
        Type::Macro(_) => panic!("Don't think I can handle this easily..."),

        // Makes no sense to use those in an OptionalStruct
        Type::Reference(_) => wtf!("reference"),
        Type::Never(_) => wtf!("never-type"),
        Type::Slice(_) => wtf!("slice"),
        Type::Ptr(_) => wtf!("pointer"),
        Type::BareFn(_) => wtf!("function pointer"),

        // Help
        Type::Verbatim(_) => todo!("Didn't get what this was supposed to be..."),
        Type::Group(_) => todo!("Not sure what to do here"),

        // Have to wildcard here but I don't want to (unneeded as long as syn doesn't break semver
        // anyway)
        _ => panic!("Open an issue please :)"),
    }
}

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
    fn new(attr: StructAttribute) -> Self {
        let new_struct_name = attr.new_struct_name;
        let default_wrapping_behavior = attr.default_wrapping;
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
                field.vis = Visibility::Public(syn::token::Pub(field.vis.span()))
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
    new_type: Option<TokenTree>,
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
        wrap: !is_type_option(&field.ty) && default_wrapping,
        new_type: None,
    };
    let indexes_to_remove = field
        .attrs
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            if a.path().is_ident(RENAME_ATTRIBUTE) {
                let args = a
                    .parse_args()
                    .expect("'{RENAME_ATTRIBUTE}' attribute expects one and only one argument (the new type to use)");
                field_attribute_data.new_type = Some(args);
                Some(i)
            } else if a.path().is_ident(SKIP_WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = false;
                Some(i)
            } else if a.path().is_ident(WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = true;
                Some(i)
            } else {
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

fn acc_assigning<'a, T: Iterator<Item=(U, &'a Vec<Attribute>)>, U: std::borrow::Borrow<V>, V: ToTokens>(
    idents_with_attrs: T,
) -> TokenStream {
    let mut acc = quote! {};
    for (ident, attrs) in idents_with_attrs {
        let ident = ident.borrow();
        let cfg_attr = attrs.iter().find(|attr| attr.path().is_ident("cfg"));
        acc = quote! {
            #acc

            #cfg_attr
            self.#ident.apply_to(&mut t.#ident);
        };
    }
    acc
}

fn generate_apply_fn(
    derive_input: &DeriveInput,
    new_struct: &DeriveInput,
) -> TokenStream {
    let orig_name = &derive_input.ident;
    let new_name = &new_struct.ident;

    let fields = match &derive_input.data {
        Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let acc = match &fields {
        Fields::Unit => unreachable!(),
        Fields::Named(fields_named) => {
            let it = fields_named.named.iter().map(|f| (f.ident.as_ref().unwrap(), &f.attrs));
            acc_assigning::<_, _, Ident>(it)
        }
        Fields::Unnamed(fields_unnamed) => {
            let it = fields_unnamed.unnamed.iter().enumerate().map(|(i, field)| {
                let i = syn::Index::from(i);
                (quote! {#i}, &field.attrs)
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
) -> TokenStream {
    let mut extra_derive = extra_derive.iter().collect::<HashSet<_>>();
    for attributes in &mut new_struct.attrs {
        let _ = attributes.parse_nested_meta(|derived_trait|
            {
                let derived_trait = derived_trait.path;
                let full_path = quote! { #derived_trait };
                extra_derive.remove(&full_path.to_string());
                Ok(())
            });
    }


    let mut acc = quote! {};
    for left_trait_to_derive in extra_derive {
        let left_trait_to_derive = format_ident!("{left_trait_to_derive}");
        acc = quote! { # left_trait_to_derive, # acc};
    }

    quote! { #[derive(#acc)] }
}

pub struct OptionalStructOutput {
    pub original: TokenStream,
    pub generated: TokenStream,
}

pub fn opt_struct(
    attr: TokenStream,
    input: TokenStream,
) -> OptionalStructOutput {
    let attr = syn::parse2::<_>(attr).unwrap();
    let global_att = GlobalAttributes::new(attr);
    let mut derive_input = syn::parse2::<DeriveInput>(input).unwrap();
    let mut new_struct = derive_input.clone();

    set_new_struct_name(global_att.new_struct_name, &mut new_struct);
    set_new_struct_fields(&mut new_struct, &global_att.field_attributes);
    let derives = get_derive_macros(&mut new_struct, &global_att.extra_derive);
    // https://github.com/rust-lang/rust/issues/65823 :(
    remove_optional_struct_attributes(&mut derive_input);
    let apply_fn_impl = generate_apply_fn(&derive_input, &new_struct);

    OptionalStructOutput {
        original: quote! { #derive_input },
        generated: quote! {
            #derives
            #new_struct

            #apply_fn_impl
        },
    }
}