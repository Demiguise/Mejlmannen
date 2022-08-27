mod client;
mod request;
mod response;

use request::PropertyMap;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Test {
    name: String,
    uri: String,
    properties: PropertyMap,
    headers: PropertyMap,
}

async fn execute_test(test: &Test) {
    println!("Running [{}]", test.name);

    let req = request::RequestBuilder::new()
        .uri(test.uri.clone())
        .test_properties(test.properties.clone())
        .header_map(test.headers.clone())
        .verb(request::Verb::GET)
        .build();

    let resp = client::execute(req).await;
    if resp.is_err() {
        panic!("Failed to make request: {}", resp.err().unwrap());
    }

    let resp = resp.unwrap();

    let body = String::from_utf8(resp.body().clone());

    println!("Got response [{:?}]", body);
}

#[tokio::main]
async fn main() {
    let mut test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file.push("tests/data/tests.mejl");

    if !test_file.exists() {
        panic!("Can't find tests/data/tests.mejl");
    }

    let data = fs::read_to_string(test_file).expect("Unable to read file");

    let json: Vec<Test> = serde_json::from_str(&data).expect("Somethi");

    for test in json.iter() {
        execute_test(test).await;
    }
}
