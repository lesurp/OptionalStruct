#![feature(custom_attribute)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::Field;
use syn::Ident;
use quote::Tokens;

#[proc_macro_derive(OptionalStruct, attributes(optional_name, optional_derive))]
pub fn optional_struct(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = create_optional_struct(&ast);
    gen.parse().unwrap()
}

fn create_optional_struct(ast: &syn::DeriveInput) -> Tokens {
    let data = parse_attributes(&ast);

    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let &syn::VariantData::Struct(ref fields) = variant_data {
            return create_non_tuple_struct(fields, data);
        }
    }

    panic!("OptionalStruct only supports non-tuple structs for now");
}

struct Data {
    orignal_struct_name: Ident,
    optional_struct_name: Ident,
    derives: Tokens,
    nested_names: HashMap<String, String>,
}

impl Data {
    fn explode(self) -> (Ident, Ident, Tokens, HashMap<String, String>) {
        (
            self.orignal_struct_name,
            self.optional_struct_name,
            self.derives,
            self.nested_names,
        )
    }
}

fn parse_attributes(ast: &syn::DeriveInput) -> Data {
    let orignal_struct_name = ast.ident.clone();
    let mut struct_name = String::from("Optional");
    struct_name.push_str(&ast.ident.to_string());
    let mut struct_name = Ident::new(struct_name);
    let mut derives = quote!{};
    let mut nested_names = HashMap::new();

    for attribute in &ast.attrs {
        match &attribute.value {
            &syn::MetaItem::Word(_) => panic!("No word attribute is supported"),
            &syn::MetaItem::NameValue(ref name, ref value) => {
                match value {
                    &syn::Lit::Str(ref name_value, _) => {
                        if name != "optional_name" {
                            nested_names.insert(name.to_string(), name_value.clone());
                        } else {

                            struct_name = Ident::new(name_value.clone())
                        }
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
                                    derives = quote!{ #derive_name, #derives }
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

    Data {
        orignal_struct_name: orignal_struct_name,
        optional_struct_name: struct_name,
        derives: derives,
        nested_names: nested_names,
    }
}

fn create_non_tuple_struct(fields: &Vec<Field>, data: Data) -> Tokens {
    let (orignal_struct_name, optional_struct_name, derives, nested_names) = data.explode();
    let (assigners, attributes) = create_assigners_attributes(&fields, nested_names);

    quote!{
        #derives
        pub struct #optional_struct_name {
            #attributes
        }

        impl #orignal_struct_name {
            pub fn apply_options(&mut self, optional_struct: #optional_struct_name) {
                #assigners 
            }
        }
    }
}

fn create_assigners_attributes(fields: &Vec<Field>, nested_names: HashMap<String, String>) -> (Tokens, Tokens) {
    let mut attributes = quote!{};
    let mut assigners = quote!{};
    for field in fields {
        let ref type_name = &field.ty;
        let ref field_name = &field.ident.clone().unwrap();
        let next_attribute;
        let next_assigner;

        let type_name_string = quote!{#type_name}.to_string();
        let type_name_string = type_name_string.split_whitespace().fold("".to_owned(), |mut type_name, token| {type_name.push_str(token); type_name});

        if type_name_string.to_string().starts_with("Option <") {
            next_attribute = quote!{ pub #field_name: #type_name>, };
            next_assigner = quote!{ self.#field_name = optional_struct.#field_name };
        } else if nested_names.contains_key(&type_name_string) {
            let type_name = Ident::new(nested_names.get(&type_name_string).unwrap().as_str());
            next_attribute = quote!{ pub #field_name: #type_name>, };
            next_assigner = quote!{ self.#field_name.apply_options(optional_struct.#field_name) };
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

    (assigners, attributes)
}
