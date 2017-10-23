extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::Field;
use syn::Ident;
use syn::Lit;
use quote::Tokens;

#[proc_macro_derive(OptionalStruct,
                    attributes(optional_name, optional_derive, opt_nested_original,
                                 opt_nested_generated))]
pub fn optional_struct(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = generate_optional_struct(&ast);
    gen.parse().unwrap()
}

fn generate_optional_struct(ast: &syn::DeriveInput) -> Tokens {
    let data = parse_attributes(&ast);

    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let &syn::VariantData::Struct(ref fields) = variant_data {
            return create_struct(fields, data);
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

fn nested_meta_item_to_ident(nested_item: &syn::NestedMetaItem) -> &Ident {
    match nested_item {
        &syn::NestedMetaItem::MetaItem(ref item) => {
            match item {
                &syn::MetaItem::Word(ref ident) => ident,
                _ => panic!("Only traits name are supported inside optional_struct"),
            }
        }
        &syn::NestedMetaItem::Literal(_) => {
            panic!("Only traits name are supported inside optional_struct")
        }
    }
}

fn create_nested_names_map(orig: Vec<Ident>, gen: Vec<Ident>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    let orig_gen = orig.iter().zip(gen);

    for (orig, gen) in orig_gen {
        if gen.to_string().is_empty() {
            map.insert(orig.to_string(), "Optional".to_owned() + &gen.to_string());
        } else {
            map.insert(orig.to_string(), gen.to_string());
        }
    }

    map
}

fn handle_list(
    name: &Ident,
    values: &Vec<syn::NestedMetaItem>,
    nested_original: &mut Vec<Ident>,
    nested_generated: &mut Vec<Ident>,
    derives: &mut Tokens,
) {
    match name.to_string().as_str() {
        "optional_derive" => {
            let mut derives_local = quote!{};
            for value in values {
                let derive_ident = nested_meta_item_to_ident(value);
                derives_local = quote!{ #derive_ident, #derives_local }
            }
            *derives = derives_local;
        }
        "opt_nested_generated" => {
            for value in values {
                let generated_nested_name = nested_meta_item_to_ident(value);
                nested_generated.push(generated_nested_name.clone());
            }
        }
        "opt_nested_original" => {
            for value in values {
                let original_nested_name = nested_meta_item_to_ident(value);
                nested_original.push(original_nested_name.clone());
            }
        }
        _ => panic!("Only optional_derive are supported"),
    };
}

fn handle_name_value(name: &Ident, value: &Lit, struct_name: &mut Ident) {
                match value {
                    &Lit::Str(ref name_value, _) => {
                        if name == "optional_name" {
                            *struct_name = Ident::new(name_value.clone())
                        } else {
                            panic!("Only optional_name is supported");
                        }
                    }
                    _ => panic!("optional_name should be a string"),
                }
}

fn parse_attributes(ast: &syn::DeriveInput) -> Data {
    let orignal_struct_name = ast.ident.clone();
    let mut struct_name = String::from("Optional");
    struct_name.push_str(&ast.ident.to_string());
    let mut struct_name = Ident::new(struct_name);
    let mut derives = quote!{};
    let mut nested_generated = Vec::new();
    let mut nested_original = Vec::new();

    for attribute in &ast.attrs {
        match &attribute.value {
            &syn::MetaItem::Word(_) => panic!("No word attribute is supported"),
            &syn::MetaItem::NameValue(ref name, ref value) => {
                handle_name_value(name, value, &mut struct_name);
            }
            &syn::MetaItem::List(ref name, ref values) => {
                handle_list(
                    name,
                    values,
                    &mut nested_original,
                    &mut nested_generated,
                    &mut derives,
                );
            }
        }
    }

    // prevent warnings if no derive is given
    derives = if derives.to_string().is_empty() {
        quote!{}
    } else {
        quote!{ #[derive(#derives)] }
    };

    Data {
        orignal_struct_name: orignal_struct_name,
        optional_struct_name: struct_name,
        derives: derives,
        nested_names: create_nested_names_map(nested_original, nested_generated),
    }
}

fn create_struct(fields: &Vec<Field>, data: Data) -> Tokens {
    let (orignal_struct_name, optional_struct_name, derives, nested_names) = data.explode();
    let (assigners, attributes, empty) = create_fields(&fields, nested_names);

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

        impl #optional_struct_name {
            pub fn empty() -> #optional_struct_name {
                #optional_struct_name {
                    #empty
                }
            }
        }
    }
}

fn create_fields(
    fields: &Vec<Field>,
    nested_names: HashMap<String, String>,
) -> (Tokens, Tokens, Tokens) {
    let mut attributes = quote!{};
    let mut assigners = quote!{};
    let mut empty = quote!{};
    for field in fields {
        let ref type_name = &field.ty;
        let ref field_name = &field.ident.clone().unwrap();
        let next_attribute;
        let next_assigner;
        let next_empty;

        let type_name_string = quote!{#type_name}.to_string();
        let type_name_string: String = type_name_string.chars().filter(|&c| c != ' ').collect();

        if type_name_string.starts_with("Option<") {
            next_attribute = quote!{ pub #field_name: #type_name, };
            next_assigner = quote!{ self.#field_name = optional_struct.#field_name; };
            next_empty = quote!{ #field_name: None, };
        } else if nested_names.contains_key(&type_name_string) {
            let type_name = Ident::new(nested_names.get(&type_name_string).unwrap().as_str());
            next_attribute = quote!{ pub #field_name: #type_name, };
            next_assigner = quote!{ self.#field_name.apply_options(optional_struct.#field_name); };
            next_empty = quote!{ #field_name: #type_name::empty(), };
        } else {
            next_attribute = quote! { pub #field_name: Option<#type_name>, };
            next_assigner =
                quote!{
                    if let Some(attribute) = optional_struct.#field_name {
                        self.#field_name = attribute;
                    }
                };
            next_empty = quote!{ #field_name: None, };
        }

        assigners = quote!{ #assigners #next_assigner };
        attributes = quote!{ #attributes #next_attribute };
        empty = quote!{ #empty #next_empty }
    }

    (assigners, attributes, empty)
}
