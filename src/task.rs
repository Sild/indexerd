extern crate tiny_http;

pub struct Task {
    raw_req: tiny_http::Request,
}

impl Task {
    pub fn new(req: tiny_http::Request) -> Self {
        Task { raw_req: req }
    }

    pub fn respond(self, body: &str) {
        let resp = tiny_http::Response::from_string(body);
        _ = self.raw_req.respond(resp);
    }
}
