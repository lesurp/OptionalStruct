# OptionalStruct
[![Build Status](https://travis-ci.org/pLesur/OptionalStruct.svg?branch=stable_channel)](https://travis-ci.org/pLesur/OptionalStruct)
[![Crates.io](https://img.shields.io/crates/v/optional_struct.svg)](https://crates.io/crates/optional_struct)

## Goal

This crate allows the user to generate a structure containing the same fields as the original struct but wrapped in Option<T>.
A method is also implemented for the original struct, `apply_options`. It consumes the generated optional_struct, and for every Some(x) field, it assigns the original structure's value with the optional_struct one.

Now that's some confusing explanation (my English skills could use some help), but basically:


```rust
#[derive(OptionalStruct)]
struct Foo {
	meow: u32,
	woof: String,
}
```

will generate:

```rust
struct OptionalFoo {
	meow: Option<u32>,
	woof: Option<String>,
}

impl Foo {
	pub fn apply_options(&mut self, optional_struct: OptionalFoo) {
		if Some(field) = optional_struct.meow {
			self.meow = field;
		}

		if Some(field) = optional_struct.woof {
			self.woof = field;
		}

	}
}
```

## Usage

You can use this to generate a configuration for you program more easily.
If you use [toml-rs](https://github.com/alexcrichton/toml-rs) to parse your config file (using serde),
you'll need to wrap your values in Option<T>, or you need them present in the config file.
With this crate, you can easily generate your whole Config struct with an Option<T> wrap for each field.
This means that if a config is missing in the file, you'll get a None.

You can then easily handle default values for your config:

```rust
impl Config {
	pub fn get_user_conf() -> OptionalConfig {
		toml::from_str<OptionalConfig>(r#"
			ip = '127.0.0.1'

			[keys]
			github = 'xxxxxxxxxxxxxxxxx'
			travis = 'yyyyyyyyyyyyyyyyy'
		    "#).unwrap()
	}
}

let mut conf = Config::get_default();
let user_conf = Config::get_user_conf();
conf.apply_options(user_conf);
```

## Features

* Option<T> inside the original structs are handled. The generated struct will have the exact same field, not an Option<Option<T>>
* You can rename the generated struct:
```rust
#[derive(OptionalStruct)]
#[optional_name = "FoorBarMeowWoof"]
```
* You can also add derives to the generated struct:
```rust
#[derive(OptionalStruct)]
#[optional_derive(Serialize, Copy, Display)]
```
* You can also nest your generated struct by mapping the original types to their new names:
```rust
#[derive(OptionalStruct)]
#[opt_nested_original(LogConfig)]
#[opt_nested_generated(OptionalLogConfig)]
struct Config {
    timeout: Option<u32>,
    log_config: LogConfig,
}

#[derive(OptionalStruct)]
struct LogConfig {
    log_file: String,
    log_level: usize,
}
```

To have multiple nested types, just include each one as a pair in order:
```rust
#[derive(OptionalStruct)]
#[opt_nested_original(LogConfig)]
#[opt_nested_generated(OptionalLogConfig)]
#[opt_nested_original(PathConfig)]
#[opt_nested_generated(OptionalPathConfig)]
struct Config {
    timeout: Option<u32>,
    log_config: LogConfig,
	path_config: PathConfig,
}

#[derive(OptionalStruct)]
struct LogConfig {
    log_file: String,
    log_level: usize,
}

#[derive(OptionalStruct)]
struct PathConfig {
    root_dir: String,
}
```

By default, the nested structures will not be optional themselves. To make those
optional, use the follow:
```rust
#[derive(OptionalStruct)]
#[opt_nested_optional = true]
#[opt_nested_original(LogConfig)]
#[opt_nested_generated(OptionalLogConfig)]
struct Config {
    timeout: Option<u32>,
    log_config: LogConfig,
}

#[derive(OptionalStruct)]
struct LogConfig {
    log_file: String,
    log_level: usize,
}
```

This will generate:
```rust
struct OptionalConfig {
    timeout: Option<u32>,
    log_config: Option<LogConfig>,
}
```

You'll find some additional examples in the tests folder (yes, I know).
