use optional_struct::*;

#[optional_struct]
struct Config {
    timeout: Option<u32>,

    #[optional_skip_wrap]
    not_optional_at_all: bool,
}

#[test]
fn test_skip_wrapping() {
    let mut config = Config {
        timeout: Some(2),
        not_optional_at_all: true,
    };

    let opt_config = OptionalConfig {
        timeout: None,
        not_optional_at_all: false,
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.timeout, Some(2));
    assert!(!config.not_optional_at_all);
}
