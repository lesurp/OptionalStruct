use optional_struct::*;

#[optional_struct]
// TODO this does not work (conflicting implementations)
// #[derive(std::debug::Debug)]
#[derive(std::hash::Hash)]
struct Config {
    delay: Option<u32>,
    path: String,
}

#[test]
fn test_apply_options() {
    let mut config = Config {
        delay: Some(2),
        path: "/var/log/foo.log".to_owned(),
    };

    let opt_config = OptionalConfig {
        delay: None,
        path: Some("/tmp/bar.log".to_owned()),
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.delay, None);
    assert_eq!(config.path, "/tmp/bar.log");
}
