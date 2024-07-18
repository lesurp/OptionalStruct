/// MRE from https://github.com/lesurp/OptionalStruct/issues/23

use optional_struct::*;

#[optional_struct]
#[derive(PartialEq, Debug)]
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

#[optional_struct]
#[derive(PartialEq, Debug, Clone)]
struct Inner(i8);

fn main() {
    let optional = OptionalFoo {
        inner1: OptionalInner(Some(1)),
        inner2: Some(OptionalInner(Some(2))),
        inner3: Some(Some(Inner(3))),
        inner4: Some(Inner(4)),
        inner5: Some(OptionalInner(Some(2))),
    };
    println!("{:?}", optional);
}
