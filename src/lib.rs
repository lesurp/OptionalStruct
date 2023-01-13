use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AttributeArgs, Data, DeriveInput, Field, Fields, Ident, Meta, NestedMeta,
    Path, Type,
};

trait Applyable<T> {
    fn apply_to(self, t: &mut T);
}

impl<T> Applyable<T> for Option<T> {
    fn apply_to(self, t: &mut T) {
        if let Some(s) = self {
            *t = s;
        }
    }
}

impl<T> Applyable<Option<T>> for Option<T> {
    fn apply_to(self, t: &mut Option<T>) {
        *t = self;
    }
}

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

fn iter_struct_fields<F: Fn(&mut Field, Option<proc_macro2::TokenTree>)>(
    the_struct: &mut DeriveInput,
    f: &F,
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
            let index_and_new_type = get_rename_attribute(field);
            if let Some((i, new_type)) = index_and_new_type {
                f(field, Some(new_type));
                field.attrs.swap_remove(i);
            } else {
                f(field, None);
            }
        }
    }
}

fn set_new_struct_fields(new_struct: &mut DeriveInput) {
    let wrap_with_option = |field: &mut Field, token_tree: Option<proc_macro2::TokenTree>| {
        let t = match token_tree {
            Some(tt) => strip_from_delimiter(&tt),
            None => {
                let t = &field.ty;
                quote! { #t }
            }
        };
        field.ty = Type::Verbatim(quote! { Option<#t> });
    };
    iter_struct_fields(new_struct, &wrap_with_option)
}

fn remove_optional_struct_attributes(original_struct: &mut DeriveInput) {
    let do_nothing = |_field: &mut Field, _token_tree: Option<proc_macro2::TokenTree>| {};
    iter_struct_fields(original_struct, &do_nothing)
}

fn path_is(p: &Path, name: &str) -> bool {
    match p.segments.len() {
        1 => p.segments[0].ident == name,
        _ => false,
    }
}

const RENAME_ATTRIBUTE: &str = "optional_rename";
fn strip_from_delimiter(token_tree: &proc_macro2::TokenTree) -> proc_macro2::TokenStream {
    match token_tree {
        proc_macro2::TokenTree::Ident(i) => quote! {#i},
        proc_macro2::TokenTree::Group(g) => {
            let tokens = g.stream().into_iter().collect::<Vec<_>>();
            if tokens.len() != 1 {
                panic!("'{RENAME_ATTRIBUTE}' attribute expects one and only one token (the new type to use)");
            }
            strip_from_delimiter(&tokens[0])
        }
        proc_macro2::TokenTree::Punct(_) => panic!("POUF"),
        proc_macro2::TokenTree::Literal(_) => {
            panic!("Token passed to '{RENAME_ATTRIBUTE}' attribute should be a type")
        }
    }
}

fn get_rename_attribute(field: &mut Field) -> Option<(usize, proc_macro2::TokenTree)> {
    field
        .attrs
        .iter()
        .enumerate()
        .find_map(|(i, a)| {
            if !path_is(&a.path, RENAME_ATTRIBUTE) {
                return None;
            }

            let mut tokens = a.tokens.clone().into_iter().collect::<Vec<_>>();
            if tokens.len() != 1 {
                panic!("'{RENAME_ATTRIBUTE}' attribute expects one and only one token (the new type to use)");
            }

            Some((i, tokens.pop().unwrap()))
        })
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
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    let mut new_struct = derive_input.clone();

    set_new_struct_name(&attr, &mut new_struct);
    set_new_struct_fields(&mut new_struct);
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
