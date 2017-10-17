extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(OptionalStruct)]
pub fn optional_struct(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = create_optional_struct(&ast);
    gen.parse().unwrap()
}

fn create_optional_struct(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let &syn::VariantData::Struct(ref fields) = variant_data {

            let mut attributes = quote! {};
            for field in fields {
                let ref type_name = &field.ty;
                let ref field_name = &field.ident.clone().unwrap();
                let next_attribute = quote! { pub Option<#type_name> #field_name, };
                attributes = quote!{ #attributes #next_attribute };
            }

            return quote!{
                struct Optional#name {
                    #attributes
                }
            }
        }
    }

    panic!("OptionalStruct only supports non-tuple structs for now");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
