use optional_struct::*;

#[optional_struct]
struct Config {
    timeout: Option<u32>,

    #[optional_skip_wrap]
    log_config: LogConfig,
}

#[optional_struct]
struct LogConfig {
    log_file: String,
    log_level: usize,
}

#[test]
fn test_apply_options_nested_skip() {
    let mut config = Config {
        timeout: Some(2),
        log_config: LogConfig {
            log_file: "/var/log/foobar.log".to_owned(),
            log_level: 3,
        },
    };

    let opt_config = OptionalConfig {
        timeout: None,
        log_config: OptionalLogConfig {
            log_file: Some("/tmp/bar.log".to_owned()),
            log_level: None,
        },
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.timeout, None);
    assert_eq!(config.log_config.log_file, "/tmp/bar.log");
    assert_eq!(config.log_config.log_level, 3);
}
