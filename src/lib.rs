extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::Ident;
use syn::Field;

#[proc_macro_derive(OptionalStruct)]
pub fn optional_struct(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = create_optional_struct(&ast);
    gen.parse().unwrap()
}

fn create_optional_struct(ast: &syn::DeriveInput) -> quote::Tokens {
    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let &syn::VariantData::Struct(ref fields) = variant_data {
            return create_non_tuple_struct(fields, &ast);
        }
    }

    panic!("OptionalStruct only supports non-tuple structs for now");
}

fn create_non_tuple_struct(fields: &Vec<Field>, ast: &syn::DeriveInput) -> quote::Tokens {
    let struct_name = &ast.ident;
    let struct_name_string = quote!{#struct_name}.to_string();
    let mut optional_struct_name = String::from("Optional");
    optional_struct_name.push_str(&struct_name_string);
    let optional_struct_name = Ident::new(optional_struct_name);

    let mut attributes = quote!{};
    let mut assigners = quote!{};
    for field in fields {
        let ref type_name = &field.ty;
        let ref field_name = &field.ident.clone().unwrap();
        let next_attribute;
        let next_assigner;

        if field_name.to_string().starts_with("Option<") {
            next_attribute = quote!{ pub #field_name: #type_name>, };
            next_assigner = quote!{ self.#field_name = optional_struct.#field_name };
        } else {
            next_attribute = quote! { pub #field_name: Option<#type_name>, };
            next_assigner =
                quote!{
                    if let Some(attribute) = optional_struct.#field_name {
                        self.#field_name = attribute;
                    }
                };
        }

        assigners = quote!{ #assigners #next_assigner };
        attributes = quote!{ #attributes #next_attribute };
    }

    quote!{
        pub struct #optional_struct_name {
            #attributes
        }

        impl #struct_name {
            pub fn apply_options(&mut self, optional_struct: &#optional_struct_name) {
                #assigners 
            }
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
