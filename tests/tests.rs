#[test]
fn json_test() {
    let json = json::parse(r#"{
        "val1": true,
        "val2": 123.45,
        "val3": [
            null, {"test": 1234, "test2": "null"}
        ],
        "val4": "word"
    }"#);

    let json = json::generate(&(json.unwrap()));

    const MAYBE_OUTPUT: &str = r#"{"val1": true, "val2": 123.45, "val3": [null, {"test": 1234, "test2": null}], "val4": "word"}"#;
    assert_eq!(json.len(), MAYBE_OUTPUT.len());
}