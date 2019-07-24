#[cfg(test)]
#[macro_use]
extern crate optional_struct;
extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(OptionalStruct)]
#[optional_derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[opt_nested_optional = true]
#[opt_nested_original(LogConfig)]
#[opt_nested_generated(OptionalLogConfig)]
#[opt_nested_original(StructureConfig)]
#[opt_nested_generated(OptionalStructureConfig)]
struct Config {
    timeout: Option<u32>,
    log_config: LogConfig,
    structure: StructureConfig,
}

#[derive(OptionalStruct)]
#[optional_derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LogConfig {
    log_file: String,
    log_level: usize,
}

#[derive(OptionalStruct)]
#[opt_nested_optional = true]
#[opt_nested_original(ModuleConfig)]
#[opt_nested_generated(OptionalModuleConfig)]
#[optional_derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StructureConfig {
    root_dir: String,
    users: ModuleConfig,
    records: ModuleConfig,
}

#[derive(OptionalStruct)]
#[optional_derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ModuleConfig {
    root_dir: String,
    enabled: bool,
}

#[test]
fn test_apply_options() {
    let mut config = create_default();
    let opt_config = "---\nstructure:\n  users:\n    enabled: false";
    let opt_config: OptionalConfig = serde_yaml::from_str(&opt_config).unwrap();

    config.apply_options(opt_config);

    assert_eq!(config.timeout, None);
    assert_eq!(config.log_config.log_file, "/var/log/foobar.log");
    assert_eq!(config.log_config.log_level, 3);
    assert_eq!(config.structure.users.enabled, false);
    assert_eq!(config.structure.records.enabled, false);
}

#[test]
fn test_apply_options_timeout() {
    let mut config = create_default();
    let opt_config = "---\ntimeout: 2";
    let opt_config: OptionalConfig = serde_yaml::from_str(&opt_config).unwrap();

    config.apply_options(opt_config);

    assert_eq!(config.timeout, Some(2));
    assert_eq!(config.log_config.log_file, "/var/log/foobar.log");
    assert_eq!(config.log_config.log_level, 3);
    assert_eq!(config.structure.users.enabled, true);
    assert_eq!(config.structure.records.enabled, false);
}

#[test]
fn test_apply_options_yaml() {
    let mut config = create_default();
    let opt_config = OptionalConfig {
        timeout: None,
        log_config: None,
        structure: Some(OptionalStructureConfig {
            root_dir: None,
            users: Some(OptionalModuleConfig {
                root_dir: None,
                enabled: Some(false),
            }),
            records: None,
        }),
    };

    config.apply_options(opt_config);

    assert_eq!(config.timeout, None);
    assert_eq!(config.log_config.log_file, "/var/log/foobar.log");
    assert_eq!(config.log_config.log_level, 3);
    assert_eq!(config.structure.users.enabled, false);
    assert_eq!(config.structure.records.enabled, false);
}

fn create_default() -> Config {
    Config {
        timeout: Some(2),
        log_config: LogConfig {
            log_file: "/var/log/foobar.log".to_owned(),
            log_level: 3,
        },
        structure: StructureConfig {
            root_dir: "/root".to_owned(),
            users: ModuleConfig {
                root_dir: "./users".to_owned(),
                enabled: true,
            },
            records: ModuleConfig {
                root_dir: "./records".to_owned(),
                enabled: false,
            },
        },
    }
}
