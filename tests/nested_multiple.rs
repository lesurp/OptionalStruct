#[macro_use]
extern crate optional_struct;

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

#[test]
fn test_apply_options() {
    let mut config = Config {
        timeout: Some(2),
        log_config: LogConfig {
            log_file: "/var/log/foobar.log".to_owned(),
            log_level: 3,
        },
        path_config: PathConfig {
            root_dir: "/root".to_owned(),
        },
    };

    let opt_config = OptionalConfig {
        timeout: None,
        log_config: OptionalLogConfig {
            log_file: Some("/tmp/bar.log".to_owned()),
            log_level: None,
        },
        path_config: OptionalPathConfig {
            root_dir: Some("/new".to_owned()),
        },
    };

    config.apply_options(opt_config);

    assert_eq!(config.timeout, None);
    assert_eq!(config.log_config.log_file, "/tmp/bar.log");
    assert_eq!(config.log_config.log_level, 3);
    assert_eq!(config.path_config.root_dir, "/new");
}
