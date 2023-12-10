use quote::quote;

use crate::opt_struct;

#[test]
fn basic_gen() {
    opt_struct(quote!(), quote!(
        struct Foo {
           bar: u8,
           baz: String
        }
    ));
}

#[test]
fn with_redundant_derive() {
    opt_struct(quote!(), quote!(
        #[derive(Debug, Clone)]
        struct Foo {
           bar: u8,
           baz: String
        }
    ));
}

#[test]
fn with_unrelated_derive() {
    opt_struct(quote!(), quote!(
        #[derive(Display)]
        struct Foo {
           bar: u8,
           baz: String
        }
    ));
}

#[test]
fn with_option() {
    opt_struct(quote!(), quote!(
        struct Foo {
           bar: Option<u8>,
        }
    ));
}
