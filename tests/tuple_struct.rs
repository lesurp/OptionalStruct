use optional_struct::*;

#[optional_struct]
struct Config(Option<u32>, String, f32);

#[test]
fn test_apply_options_tuple_struct() {
    let opt_config = OptionalConfig::default();
    assert_eq!(opt_config.0, None);
    assert_eq!(opt_config.1, None);
    assert_eq!(opt_config.2, None);
}
