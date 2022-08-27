#[derive(Debug)]
pub struct Response {
    status: u16,
    body: Vec<u8>,
}

impl Response {
  pub fn body(&self) -> &Vec<u8> {
    &self.body
  }

  pub fn status(&self) -> u16 {
    self.status
  }
}

pub struct ResponseBuilder {
    status: u16,
    body: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            status: 0,
            body: Vec::new(),
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

    pub fn build(self) -> Response {
        Response {
            status: self.status,
            body: self.body,
        }
    }
}
