use optional_struct::*;

#[optional_struct]
struct Foo {
    bar: Option<u8>,
    #[optional_wrap]
    baz: Option<u8>
}

#[test]
fn test_force_wrap_option() {
    let _opt_foo = OptionalFoo {
        bar: Some(1),
        baz: Some(Some(2)),
    };
}
