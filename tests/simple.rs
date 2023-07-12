use optional_struct::*;

#[optional_struct]
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
        delay: Some(Some(5)),
        path: Some("/tmp/bar.log".to_owned()),
        percentage: Some(42.24),
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.delay, Some(5));
    assert_eq!(config.path, "/tmp/bar.log");
    assert_eq!(config.percentage, 42.24);
}
