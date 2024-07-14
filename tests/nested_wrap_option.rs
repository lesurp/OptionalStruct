/// MRE from https://github.com/lesurp/OptionalStruct/issues/23

use optional_struct::*;

#[optional_struct]
#[derive(PartialEq, Debug)]
struct Foo {
    #[optional_skip_wrap]
    #[optional_rename(OptionalInner)]
    inner: Option<Inner>,
}

#[optional_struct]
#[derive(PartialEq, Debug, Clone)]
struct Inner(i8);

fn main() {
    let optional = OptionalFoo {
        inner: OptionalInner(Some(1))
    };
    println!("{:?}", optional);
}
