mod client;
mod request;
mod response;

use request::PropertyMap;

fn main() {
    // User might have a global set of properties
    let mut global_kv = PropertyMap::new();
    global_kv.insert("Global_Key".to_owned(), "Global Value".to_owned());
    global_kv.insert("Local_Key".to_owned(), "Global Value".to_owned());
    global_kv.insert("Test_Key".to_owned(), "Global Value".to_owned());

    // Then a local set of overrides or new values
    let mut local_kv = PropertyMap::new();
    local_kv.insert("Local_Key".to_owned(), "Local Value".to_owned());
    local_kv.insert("Local_Name".to_owned(), "Barry".to_owned());
    local_kv.insert("Test_Key".to_owned(), "Local Value".to_owned());

    // Finally a set of per-test overrides or new values
    let mut test_kv = PropertyMap::new();
    test_kv.insert("Test_Key".to_owned(), "Test Value".to_owned());
    test_kv.insert("version".to_owned(), "v1".to_owned());
    test_kv.insert("endpoint".to_owned(), "dothing".to_owned());

    // Some URL we want to hit
    let url = "localhost:8043/api/{{version}}/{endpoint}/{not_found}".to_owned();

    let req = request::RequestBuilder::new()
        .url(url)
        .global_properties(global_kv)
        .local_properties(local_kv)
        .test_properties(test_kv)
        .header("AUTHENTICATION".to_owned(), "Your mum".to_owned())
        .verb(request::Verb::GET)
        .build();

    let resp = client::execute(req);
    if resp.is_err() {
        panic!("Failed to make request: {}", resp.err().unwrap());
    }

    println!("Got response {:?}", resp.unwrap());
}
