use optional_struct::*;

#[optional_struct]
#[derive(Eq, PartialEq, Debug)]
struct Foo {
    paf: u16,
    bar: Option<u8>,
    #[optional_wrap]
    baz: Option<char>,
    #[optional_rename(OptionalMiaou)]
    #[optional_wrap]
    miaou: Miaou,
}

#[optional_struct]
#[derive(Eq, PartialEq, Debug)]
struct Miaou {
    a: i8,
    b: i16,
}

#[test]
fn test_builder() {
    let default = Foo {
        paf: 12,
        bar: None,
        baz: Some('a'),
        miaou: Miaou {
            a: 1,
            b: -1,
        },
    };

    let first = OptionalFoo {
        paf: Some(42),
        bar: Some(7),
        baz: Some(None),
        miaou: None,
    };

    let second = OptionalFoo {
        paf: Some(24),
        bar: None,
        baz: Some(Some('c')),
        miaou: Some(OptionalMiaou {
            a: Some(2),
            b: None,
        }),
    };

    let collapsed = first.apply(second).build(default);
    assert_eq!(collapsed, Foo {
        paf: 24,
        bar: Some(7),
        baz: Some('c'),
        miaou: Miaou { a: 2, b: -1 },
    });
}