fn main() {
    // let json = json::parse("
    // {
    //     \"value\": 1234.5 ,
    //     \"value2\": \"abcd\",
    // }
    // ").unwrap();

    // for (key, value) in &json {
    //     println!("{}: {:?}", key, value);
    // }

    // let json = json::parse("{
    //     \"val1\": true,
    //     \"val2\": 123.45,
    //     \"val3\": \"word\",
    // }");

    let json = json::parse(r#"{
        "val1": true,
        "val2": "error",
        "val3": [false, "false", {"true": false}]
    }"#).unwrap();

    println!("{:?}", json);
    println!("{}", json::generate(&json));
}