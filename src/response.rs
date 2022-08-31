use crate::common::StringMap;

#[derive(Debug)]
pub struct Response {
    status: u16,
    body: Vec<u8>,
    headers: StringMap
}

impl Response {
  pub fn body(&self) -> &Vec<u8> {
    &self.body
  }

  pub fn status(&self) -> u16 {
    self.status
  }

  pub fn headers(&self) -> &StringMap {
    &self.headers
  }
}

pub struct ResponseBuilder {
    status: u16,
    body: Vec<u8>,
    headers: StringMap
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            status: 0,
            body: Vec::new(),
            headers: StringMap::new()
        }
    }

    pub fn body(mut self, body: Vec<u8>) -> ResponseBuilder {
        self.body = body;
        self
    }

    pub fn status(mut self, status: u16) -> ResponseBuilder {
        self.status = status;
        self
    }

    pub fn headers(mut self, headers: StringMap) -> ResponseBuilder {
        self.headers = headers;
        self
    }

    pub fn build(self) -> Response {
        Response {
            status: self.status,
            body: self.body,
            headers: self.headers
        }
    }
}
