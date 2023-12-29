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
#[optional_struct]
struct Config {
    #[optional_wrap]
    delay: Option<u32>,
    path: String,
    percentage: f32,
}

#[test]
fn test_apply_double_wrap() {
    let mut config = Config {
        delay: Some(2),
        path: "/var/log/foo.log".to_owned(),
        percentage: 3.12,
    };

    let opt_config = OptionalConfig {
        delay: Some(None),
        path: Some("/tmp/bar.log".to_owned()),
        percentage: Some(42.24),
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.delay, None);
    assert_eq!(config.path, "/tmp/bar.log");
    assert_eq!(config.percentage, 42.24);
}
