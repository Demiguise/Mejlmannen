use crate::response;

use anyhow::Result;

use hyper::Client;

/*
  Abstracts away the underlying REST client implementation, as we
  don't really care about it. Just execute the request and hand back
  errors or the response.
*/
mod request_converter {
    use crate::request;
    use anyhow::{Context, Result};
    use hyper::{Body, Request};

    fn convert_verb(verb: request::Verb) -> hyper::Method {
        match verb {
            request::Verb::GET => hyper::Method::GET,
            request::Verb::POST => hyper::Method::POST,
            request::Verb::DELETE => hyper::Method::DELETE,
        }
    }

    pub fn convert(req: request::Request) -> Result<Request<Body>> {
        Request::builder()
            .method(convert_verb(req.verb()))
            .uri(req.replaced_uri())
            .body(Body::from(req.body().clone()))
            .with_context(|| "Failed?") // TODO: Better message
    }
}

mod response_converter {
    use crate::response;
    use anyhow::Result;
    use hyper::{Body, Response};

    pub async fn convert(resp: Response<Body>) -> Result<response::Response> {
        let status_code = resp.status();
        let buf = hyper::body::to_bytes(resp).await.expect("Something");

        let converted = response::ResponseBuilder::new()
            .status(status_code.as_u16())
            .body(buf.to_vec())
            .build();

        Ok(converted)
    }
}

pub async fn execute(req: crate::request::Request) -> Result<response::Response> {
    let converted = request_converter::convert(req)?;

    println!("Making request with: {:?}", converted);
    let client = Client::new();
    let resp = client.request(converted).await;

    match resp {
        Ok(result) => response_converter::convert(result).await,
        Err(e) => panic!(""),
    }
}
