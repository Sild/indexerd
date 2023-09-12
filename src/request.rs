extern crate tiny_http;

pub struct Request {
    raw_req: tiny_http::Request,
}

impl Request {
    pub fn new(req: tiny_http::Request) -> Self {
        Request { raw_req: req }
    }

    pub fn respond(self, body: &str) {
        let resp = tiny_http::Response::from_string(body);
        _ = self.raw_req.respond(resp);
    }
}
