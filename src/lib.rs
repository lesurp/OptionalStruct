use syn::{
    parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, Ident, Meta, NestedMeta, Path,
    Type,
};

use quote::{format_ident, quote};

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
        Type::Verbatim(_) | Type::Group(_) => todo!("Didn't get what this was supposed to be..."),

        // Have to wildcard here but I don't want to (unneeded as long as syn doesn't break semver
        // anyway)
        _ => panic!("Open an issue please :)"),
    }
}

fn get_optional_struct_name(attr: &AttributeArgs) -> Option<String> {
    attr.iter()
        .filter_map(|ns| {
            if let NestedMeta::Meta(m) = ns {
                Some(m)
            } else {
                None
            }
        })
        .filter_map(|m| match m {
            Meta::Path(p) => Some(p),
            Meta::NameValue(_) | Meta::List(_) => None,
        })
        .map(|p| {
            p.segments
                .last()
                .expect("How can we have an empty path here?")
                .ident
                .to_string()
        })
        .next()
}

fn set_new_struct_name(attr: &AttributeArgs, new_struct: &mut DeriveInput) {
    let new_struct_name = get_optional_struct_name(attr)
        .unwrap_or_else(|| "Optional".to_owned() + &new_struct.ident.to_string());

    new_struct.ident = Ident::new(&new_struct_name, new_struct.ident.span());
}

fn set_new_struct_fields(new_struct: &mut DeriveInput) {
    let data_struct = match &mut new_struct.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("OptionalStruct only works for structs :)"),
    };

    let fields = match &mut data_struct.fields {
        Fields::Unnamed(f) => &mut f.unnamed,
        Fields::Named(f) => &mut f.named,
        Fields::Unit => unreachable!("A struct cannot have simply a unit field?"),
    };

    for field in fields {
        if !is_type_option(&field.ty) {
            // required because quote chokes on #a.b
            let t = &field.ty;
            field.ty = Type::Verbatim(quote! { Option<#t> });
        }
    }
}

fn acc_assigning<T: std::iter::Iterator<Item = U>, U: std::borrow::Borrow<Ident>>(
    idents: T,
) -> proc_macro2::TokenStream {
    let mut acc = quote! {};
    for field in idents {
        let field = field.borrow();
        acc = quote! {
            #acc
            if let Some(val) = o.#field {
                // TODO: check if self.#field is not an option too!
                self.#field = val;
            }
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

    quote! {
        impl #orig_name {
            pub fn apply_options(&mut self, o: &#new_name) {
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
    let derive_input = parse_macro_input!(input as DeriveInput);
    let mut new_struct = derive_input.clone();

    set_new_struct_name(&attr, &mut new_struct);
    set_new_struct_fields(&mut new_struct);
    let apply_fn_impl = generate_apply_fn(&derive_input, &new_struct);

    let output = quote! {
        #derive_input

        #[derive(Default, Clone, PartialEq)]
        #new_struct

        #apply_fn_impl
    };
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(
    OptionalStruct,
    attributes(
        optional_name,
        optional_derive,
        opt_nested_original,
        opt_nested_generated
    )
)]
pub fn optional_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    optional_struct(proc_macro::TokenStream::new(), input)
}
