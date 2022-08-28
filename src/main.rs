mod client;
mod request;
mod response;

use anyhow::{Context, Result};
use request::PropertyMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Test {
    name: String,
    uri: String,
    properties: PropertyMap,
    headers: PropertyMap,
}

type TestMap = HashMap<String, Vec<Test>>;

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

fn load_directory(root_dir: &PathBuf, current_dir: &PathBuf, map: &mut TestMap) -> Result<()> {
    let contents = current_dir.read_dir()?;
    for item in contents {
        let path = item
            .with_context(|| format!("Failed to read file in {}", current_dir.display()))?
            .path();

        if path.is_dir() {
            // Recurse down that file path
            load_directory(root_dir, &path, map)?;
            continue;
        }

        let ext = path.extension();
        if ext.is_some() && ext.unwrap() == "mejl" {
            // Ignore mejl configuration files
            continue;
        }

        // Get the path we're currently looking at at, without the root_dir attached
        // At the same time, convert it to a string
        let test_path = current_dir
            .strip_prefix(root_dir)
            .with_context(|| "Failed to get test path")?
            .to_owned()
            .into_os_string()
            .into_string()
            .expect("Failed to create test_path");

        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read data from {}", path.display()))?;
        match serde_json::from_str(&data) {
            Ok(json) => {
                map.insert(test_path, json);
            }
            Err(e) => {
                println!("Failed to parse JSON from {} [{}]", path.display(), e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut tests = HashMap::new();

    load_directory(&test_dir, &test_dir, &mut tests).expect("Failed to load paths from directory");

    for (path, tests) in tests.iter() {
        println!("Running tests from {}", path);
        for test in tests.iter() {
            execute_test(test).await;
        }
    }
}
