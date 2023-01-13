use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AttributeArgs, Data, DeriveInput, Field, Fields, Ident, Meta, NestedMeta,
    Path, Type,
};

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
    default_wrapping_behavior: bool,
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
            default_wrapping_behavior,
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

fn iter_struct_fields(
    the_struct: &mut DeriveInput,
    apply_attribute_metadata: bool,
    default_wrapping: bool,
) {
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
        if !is_type_option(&field.ty) {
            let field_meta_data = extract_relevant_attributes(field, default_wrapping);
            if apply_attribute_metadata {
                field_meta_data.apply_to_field(field);
            }
        }
    }
}

fn set_new_struct_fields(new_struct: &mut DeriveInput, default_wrapping: bool) {
    iter_struct_fields(new_struct, true, default_wrapping)
}

fn remove_optional_struct_attributes(original_struct: &mut DeriveInput) {
    // Last boolean isn't actually used but w/e
    iter_struct_fields(original_struct, false, true)
}

fn path_is(p: &Path, name: &str) -> bool {
    match p.segments.len() {
        1 => p.segments[0].ident == name,
        _ => false,
    }
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
            if path_is(&a.path, RENAME_ATTRIBUTE) {
                let mut tokens = a.tokens.clone().into_iter().collect::<Vec<_>>();
                if tokens.len() != 1 {
                    panic!("'{RENAME_ATTRIBUTE}' attribute expects one and only one token (the new type to use)");
                }

                field_attribute_data.new_type = Some(tokens.pop().unwrap());
                Some(i)
            }
            else if path_is(&a.path, SKIP_WRAP_ATTRIBUTE) {
                field_attribute_data.wrap = false;
                Some(i)
            }
            else if path_is(&a.path, WRAP_ATTRIBUTE) {
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

fn acc_assigning<T: std::iter::Iterator<Item = U>, U: std::borrow::Borrow<Ident>>(
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
            acc_assigning(it)
        }
        Fields::Unnamed(fields_unnamed) => {
            let it = fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("{i}"));
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
    set_new_struct_fields(&mut new_struct, global_att.default_wrapping_behavior);
    // https://github.com/rust-lang/rust/issues/65823 :(
    remove_optional_struct_attributes(&mut derive_input);
    let apply_fn_impl = generate_apply_fn(&derive_input, &new_struct);

    let output = quote! {
        #derive_input

        #[derive(Default, Clone, PartialEq, Debug)]
        #new_struct

        #apply_fn_impl
    };
    proc_macro::TokenStream::from(output)
}
