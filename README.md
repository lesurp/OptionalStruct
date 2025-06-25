# OptionalStruct
[![Crates.io](https://img.shields.io/crates/v/optional_struct.svg)](https://crates.io/crates/optional_struct)

## Quick-start

From the `tests/builder.rs` file:

```rust
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
```

## Goal

Since rust does not have default arguments, and some tools are strict when
deserializing data (e.g. serde), missing configuration values can be quite
frustrating to deal with. For example:

```rust
#[derive(Deserialize)]
struct Config {
    log_file: PathBuf,
}
```

If we read the configuration from a file, and the `log_file` is not specified,
serde will fail to create the struct. While serde [offers ways to set the
default value for a field](https://serde.rs/attr-default.html) with e.g.

```rust
#[derive(Deserialize)]
struct Config {
    #[serde(default = "get_next_log_filename")]
    log_file: PathBuf,
}
```

there are obvious limitations. This crate aims to fill this gap by allowing
optional values, and providing an easy way to apply values obtained from
different sources to construct our configuration.

With `optional_struct`, one can define the required
configuration as it shall be used and only use the generated struct
to handle configuration/missing values/default values.


## How

The macro `optional_struct` generates a structure containing the same fields as the one it was tagged on, but wrapped by an `Option`.
A function on the new structure allows applying its values to the original one
(if the `Option`s are not `None`). This can be called multiple times, to apply
configuration from different source, while giving the caller complete control
over how to set the values, since the generated struct can be easily manipulated
and passed around before constructing the final configuration.

## Features

1. Rename the generated struct:

```rust
#[optional_struct(HeyU)]
struct Config();

fn main() {
    let me = HeyU();
}
```

2. Handle recursive types:

```rust
#[optional_struct]
struct Foo {
    // Replaces Option<Bar> with OptionalBar
    // To generate Option<OptionalBar> instead, add an extra #[optional_wrap]
    // as described later
    #[optional_rename(OptionalBar)]
    bar: Bar,
}
```

3. Handle `Option`s in the original struct (by ignoring them):

```rust
#[optional_struct]
struct Foo {
    bar: Option<u8>,
}

fn main() {
    let opt_f = OptionalFoo { bar: Some(1) };
}
```

4. Force wrapping (or not) of fields:

```rust
#[optional_struct]
struct Foo {
    #[optional_skip_wrap]
    bar: char,

    // Useless here since we wrap by default
    #[optional_wrap]
    baz: bool,
}

fn main() {
    let opt_f = OptionalFoo { bar: 'a', baz: Some(false) };
}
```

5. Change the default wrapping behavior:

```rust
#[optional_struct(OptionalFoo, false)]
struct Foo {
    bar: u8,
    #[optional_wrap]
    baz: i8,
}

fn main() {
    let opt_f = OptionalFoo { bar: 1, baz: None };
}
```

6. Add serde's `skip_serializing_if = "Option::is_none"` attribute to generated
struct

By adding the attribute `#[optional_serde_skip_none]` to a field, the generated
struct will have the same field tagged `#[serde(skip_serializing_if = "Option::is_none")]`.
This attribute makes serde skip fields entirely if the value of the `Option` is
none (rather than saving e.g. `"value" = null` if serializing to json).

7. Skip a field entirely:

```rust
#[optional_struct]
#[derive(Default)]
struct Foo {
    #[optional_skip]
    bar: char,
    baz: bool,
}

fn main() {
    let opt_f = OptionalFoo { baz: Some(false) };
}
```

If the `optional_skip` attribute is used, `Default` is used to fill in missing fields in the ``TryFrom` implementation.   Therefore Default must be implemented on the struct.

## `apply`, `build`, and `try_build`

Those three functions are used to build the final version of the structure, by
collapsing the values "on the left".

The signatures of the functions are (in pseudo-code):

```rust
impl OptionalStruct {
    fn build(self, s: Struct) -> Struct;
    fn try_build(self) -> Result<Struct, OptionalStruct>;
    fn apply(self, other: OptionalStruct) -> OptionalStruct;
}
```

What those functions do:

1. `build` takes a real `Struct` and sets all its field based on which fields
   are set in `OptionalStruct`. Missing fields are left alone. `Option` fields
   with a force-wrap attributes will NOT overwrite the value e.g. `Some(1)` will
   not overwrite `Some(2)` (see the initial example for a concrete situation.

2. `try_build` tries to build a whole `Struct` from the `OptionalStruct`,
   returning either an `Ok(Struct)` if things went well,
   or the initial `OptionalStruct` in the `Err(OptionalStruct)` in case things were missing.

3. `apply` takes an `OptionalStruct` as a parameter and applies its fields to
   the *left* (i.e. `self`). If `self` and `other` both define something, the value
   from `other` is taken. If `self` defines something but not `other`, the value
   is preserved. Naturally, if `self` does not define something but `other` does,
   this value is used.
