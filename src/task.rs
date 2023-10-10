extern crate tiny_http;

use crate::config;
use crate::data::store::Store;
use tiny_http::Header;

pub struct HttpTask {
    pub raw_req: tiny_http::Request,
}

impl HttpTask {
    pub fn new(req: tiny_http::Request) -> Self {
        HttpTask { raw_req: req }
    }

    pub fn respond_html(self, body: &str) {
        let mut resp = tiny_http::Response::from_string(body);
        resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
        _ = self.raw_req.respond(resp);
    }

    pub fn respond_bin(self, body: &str) {
        let mut resp = tiny_http::Response::from_string(body);
        resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"binary"[..]).unwrap());
        _ = self.raw_req.respond(resp);
    }
}

pub struct TaskContext<'a> {
    pub store: &'a Store,
    pub config: &'a config::Worker,
}

pub struct AdminTask<'a> {
    pub http_task: HttpTask,
    pub context: TaskContext<'a>,
}

pub struct SearchTask<'a> {
    pub http_task: HttpTask,
    pub context: TaskContext<'a>,
}
