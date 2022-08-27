#[derive(Debug)]
pub struct Response {
    status: u16,
    body: String,
}

pub struct ResponseBuilder {
    status: u16,
    body: String,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            status: 0,
            body: String::new(),
        }
    }

    pub fn body(mut self, body: String) -> ResponseBuilder {
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
