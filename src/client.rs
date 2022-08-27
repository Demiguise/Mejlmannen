use crate::request::Request;
use crate::response::Response;

use anyhow::Result;

/*
  Abstracts away the underlying REST client implementation, as we
  don't really care about it. Just execute the request and hand back
  errors or the response.
*/

pub fn execute(req: Request) -> Result<Response> {
    Ok(Response {})
}
