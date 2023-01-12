#[macro_use]
extern crate optional_struct;

#[optional_struct]
struct Config {
    delay: Option<u32>,
    path: String,
    percentage: f32,
}

#[test]
fn test_apply_options() {
    let opt_config = OptionalConfig::default();

    assert_eq!(opt_config.delay, None);
    assert_eq!(opt_config.path, None);
    assert_eq!(opt_config.percentage, None);
}
