use optional_struct::optional_struct;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[optional_struct]
#[derive(Serialize, Deserialize)]
struct Foo {
    #[optional_serde_skip_none]
    bar: Option<u32>,
    #[optional_serde_skip_none]
    baz: String,
    meow: f32,
}

#[test]
fn test_serde_skip() {
    let opt = OptionalFoo {
        bar: None,
        baz: None,
        meow: Some(0.5),
    };

    let serialized = serde_json::to_value(opt).unwrap();
    assert_eq!(
        serialized,
        json!(
            {
                "meow": 0.5
            }
        )
    );
}
