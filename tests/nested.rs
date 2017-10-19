#![feature(custom_attribute)]

#[macro_use]
extern crate optional_struct;

#[derive(OptionalStruct)]
#[LogConfig = "OptionalLogConfig"]
struct Config {
    timeout: Option<u32>,
    log_config: LogConfig,
}

#[derive(OptionalStruct)]
struct LogConfig {
    log_file: String,
    log_level: usize,
}

#[test]
fn test_apply_options() {
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

    config.apply_options(opt_config);

    assert_eq!(config.timeout, None);
    assert_eq!(config.log_config.log_file, "/tmp/bar.log");
    assert_eq!(config.log_config.log_level, 3);
}
