#[macro_use]
extern crate optional_struct;

#[derive(OptionalStruct)]
#[optional_derive(Debug, Clone, PartialEq)]
struct Config {
    delay: Option<u32>,
    path: String,
    percentage: f32,
}

#[test]
fn test_apply_options() {
    let mut config = Config {
        delay: Some(2),
        path: "/var/log/foo.log".to_owned(),
        percentage: 3.12,
    };

    let opt_config = OptionalConfig {
        delay: None,
        path: Some("/tmp/bar.log".to_owned()),
        percentage: Some(42.24),
    };

    let cloned = opt_config.clone();

    assert_eq!(opt_config, cloned);

    config.apply_options(opt_config);
    assert_eq!(config.delay, None);
    assert_eq!(config.path, "/tmp/bar.log");
    assert_eq!(config.percentage, 42.24);
}
