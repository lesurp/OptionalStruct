use optional_struct::*;

fn generic_stuff_with_optional_struct<A: Applicable>(opt: A, base: A::Base) -> A::Base {
    opt.build(base)
}

#[optional_struct]
struct Config {
    delay: Option<u32>,
    path: String,
    percentage: f32,
}

#[test]
fn test_apply_options() {
    let config = Config {
        delay: Some(2),
        path: "/var/log/foo.log".to_owned(),
        percentage: 3.12,
    };

    let opt_config = OptionalConfig {
        delay: None,
        path: Some("/tmp/bar.log".to_owned()),
        percentage: Some(42.24),
    };

    let config = generic_stuff_with_optional_struct(opt_config, config);

    assert_eq!(config.delay, Some(2));
    assert_eq!(config.path, "/tmp/bar.log");
    assert_eq!(config.percentage, 42.24);
}
