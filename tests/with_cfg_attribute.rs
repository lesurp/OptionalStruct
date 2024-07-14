//#![feature(stmt_expr_attributes)]
use optional_struct::*;

#[optional_struct]
struct Foo {
    #[cfg(all())]
    bar: u8,
    #[cfg(any())]
    baz: u8,
}

#[test]
fn test_match_cfg_attributes() {
    let mut foo = Foo { bar: 1 };

    let opt_foo = OptionalFoo { bar: Some(1) };
    opt_foo.apply_to(&mut foo);
}
