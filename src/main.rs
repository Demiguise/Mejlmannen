mod client;
mod common;
mod extractor;
mod request;
mod response;

use anyhow::{Context, Result};
use common::StringMap;
use request::Request;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Collection {
    name: String,
    requests: Vec<Request>,
}

type CollectionMap = HashMap<PathBuf, Collection>;

async fn execute_request(request: &Request, cached_properties: &mut StringMap) {
    let resp = client::execute(request, &cached_properties).await;
    if resp.is_err() {
        panic!("Failed to make request: {}", resp.err().unwrap());
    }

    let resp = resp.unwrap();

    if resp.status() != 200 {
        println!("Request failed with code {}", resp.status());
        return;
    }

    match extractor::extract(request.extract(), &resp) {
        Ok(props) => {
            cached_properties.extend(props);
        }
        Err(e) => {
            println!("Failed to extract properties: {}", e);
            return;
        }
    }

    let body = String::from_utf8(resp.body().clone());

    println!("Got response [{:?}]", body);
}

// Run through the collection and make load any files needed by the requests
fn evaluate_collection(collection: &mut Collection, working_directory: &PathBuf) -> Result<()> {
    for req in collection.requests.iter_mut() {
        match req.update_body(working_directory) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to update body for {} [{}]", req.uri(), e);
            }
        }
    }
    Ok(())
}

fn load_directory(
    root_dir: &PathBuf,
    current_dir: &PathBuf,
    map: &mut CollectionMap,
) -> Result<()> {
    let contents = current_dir.read_dir()?;
    for content in contents {
        let item = content
            .with_context(|| format!("Failed to read file in {}", current_dir.display(),))?;

        let path = item.path();
        let name = item.file_name().into_string().unwrap(); // TODO: Error handling here

        if path.is_dir() {
            if name.starts_with("_") {
                continue;
            }

            // Directories with `_` are ignored as they probably contain data and such.
            // If not, then recurse down that file path
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
            .to_owned();

        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read data from {}", path.display()))?;
        match serde_json::from_str(&data) {
            Ok(mut json) => match evaluate_collection(&mut json, current_dir) {
                Ok(_) => {
                    map.insert(test_path, json);
                }
                Err(e) => {
                    println!("Failed to evaluate Collction {} [{}]", json.name, e);
                }
            },
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
    let mut collections = HashMap::new();

    load_directory(&test_dir, &test_dir, &mut collections)
        .expect("Failed to load paths from directory");

    let mut cached_properties = StringMap::new();
    for (path, collection) in collections.iter() {
        println!("Running tests for {}/{}", path.display(), collection.name);
        for req in collection.requests.iter() {
            execute_request(req, &mut cached_properties).await;
        }
    }
}
