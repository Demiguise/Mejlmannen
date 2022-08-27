mod client;
mod request;
mod response;

use request::PropertyMap;

#[tokio::main]
async fn main() {
    // Finally a set of per-test overrides or new values
    let mut test_kv = PropertyMap::new();
    test_kv.insert("endpoint".to_owned(), "ip".to_owned());

    // Some URL we want to hit
    let uri = "http://httpbin.org/{endpoint}".to_owned();

    let req = request::RequestBuilder::new()
        .uri(uri)
        .test_properties(test_kv)
        .header("AUTHENTICATION".to_owned(), "Your mum".to_owned())
        .verb(request::Verb::GET)
        .build();

    let resp = client::execute(req).await;
    if resp.is_err() {
        panic!("Failed to make request: {}", resp.err().unwrap());
    }

    println!("Got response {:?}", resp.unwrap());
}
