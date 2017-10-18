extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::Ident;
use syn::Field;

#[proc_macro_derive(OptionalStruct, attributes(optional_name, optional_derive))]
pub fn optional_struct(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = create_optional_struct(&ast);
    gen.parse().unwrap()
}

fn create_optional_struct(ast: &syn::DeriveInput) -> quote::Tokens {
    let (struct_name, derives) = parse_attributes(&ast);

    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let &syn::VariantData::Struct(ref fields) = variant_data {
            return create_non_tuple_struct(fields, struct_name, derives);
        }
    }

    panic!("OptionalStruct only supports non-tuple structs for now");
}

fn parse_attributes(ast: &syn::DeriveInput) -> (syn::Ident, quote::Tokens) {
    let mut struct_name = ast.ident.clone();
    let mut derives = quote!{};

    for attribute in &ast.attrs {
        match &attribute.value {
            &syn::MetaItem::Word(_) => panic!("No word attribute is supported"),
            &syn::MetaItem::NameValue(ref name, ref value) => {
                if name != "optional_name" {
                    panic!("Only optional_name is supported");
                }

                match value {
                    &syn::Lit::Str(ref name_value, _) => {
                        struct_name = syn::Ident::new(name_value.clone())
                    }
                    _ => panic!("optional_name should be a string"),
                }
            }
            &syn::MetaItem::List(ref name, ref values) => {
                if name != "optional_derive" {
                    panic!("Only optional_derive are supported");
                }

                for value in values {
                    match value {
                        &syn::NestedMetaItem::MetaItem(ref item) => {
                            match item {
                                &syn::MetaItem::Word(ref derive_name) => {
                                    derives = quote!{ #derives, #derive_name }
                                }
                                _ => {
                                    panic!("Only traits name are supported inside optional_struct")
                                }
                            }
                        }
                        &syn::NestedMetaItem::Literal(_) => {
                            panic!("Only traits name are supported inside optional_struct")
                        }
                    }
                }
            }
        }
    }

    derives = quote!{ #[derive(#derives)] };
    (struct_name, derives)
}

fn create_non_tuple_struct(
    fields: &Vec<Field>,
    struct_name: syn::Ident,
    derives: quote::Tokens,
) -> quote::Tokens {
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
        #derives
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
