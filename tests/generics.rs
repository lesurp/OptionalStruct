use optional_struct::*;

#[optional_struct]
struct GenericConfig<T: std::fmt::Debug, V> {
    value_t: T,
    value_v: V,
}

#[test]
fn test_apply_options() {
    let mut config = GenericConfig {
        value_t: 3.0,
        value_v: "foo",
    };

    let opt_config = OptionalGenericConfig {
        value_t: None,
        value_v: Some("bar"),
    };

    opt_config.apply_to(&mut config);

    assert_eq!(config.value_t, 3.0);
    assert_eq!(config.value_v, "bar");
}
