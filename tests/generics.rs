#[macro_use]
extern crate optional_struct;

#[derive(OptionalStruct)]
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

    config.apply_options(opt_config);

    assert_eq!(config.value_t, 3.0);
    assert_eq!(config.value_v, "bar");
}
