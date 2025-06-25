use quote::quote;

use crate::opt_struct::opt_struct;

#[test]
fn basic_gen() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_nested() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                #[optional_rename(OptionalBar)]
                bar: Bar,
            }
        ),
    );
}

#[test]
fn with_skip() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                #[optional_skip]
                bar: Bar,
            }
        ),
    );
}

#[test]
fn with_serde_skip() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                #[optional_serde_skip_none]
                baz: Baz,
                #[optional_serde_skip_none]
                bar: Option<Bar>,
            }
        ),
    );
}

#[test]
fn with_redundant_derive() {
    opt_struct(
        quote!(),
        quote!(
            #[derive(Debug, Clone)]
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_unrelated_derive() {
    opt_struct(
        quote!(),
        quote!(
            #[derive(Display)]
            struct Foo {
                bar: u8,
                baz: String,
            }
        ),
    );
}

#[test]
fn with_option() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                bar: Option<u8>,
            }
        ),
    );
}

#[test]
fn with_cfg_attributes() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                #[cfg(all())]
                bar: u8,
                #[cfg(any())]
                baz: u8,
            }
        ),
    );
}

#[test]
fn with_nested_option_skip_wrap() {
    opt_struct(
        quote!(),
        quote!(
            struct Foo {
                #[optional_skip_wrap]
                #[optional_rename(OptionalInner)]
                inner1: Option<Inner>,

                #[optional_rename(OptionalInner)]
                #[optional_wrap]
                inner2: Option<Inner>,

                #[optional_wrap]
                inner3: Option<Inner>,

                #[optional_skip_wrap]
                inner4: Option<Inner>,

                #[optional_wrap]
                #[optional_rename(OptionalInner)]
                inner5: Option<Inner>,
            }
        ),
    );
}
