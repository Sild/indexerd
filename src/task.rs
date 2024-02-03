extern crate tiny_http;

use crate::config;
use crate::data::store::Store;
use anyhow::Result;
use tiny_http::Header;

pub struct HttpTask {
    raw_req: tiny_http::Request,
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
    pub search_request: Result<crate::request::search_request::SearchRequest>,
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

    pub fn _respond_bin(self, body: &str) {
        let mut resp = tiny_http::Response::from_string(body);
        resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"binary"[..]).unwrap());
        _ = self.raw_req.respond(resp);
    }

    pub fn url(&self) -> &str {
        return self.raw_req.url();
    }
}

impl<'a> AdminTask<'a> {
    pub fn new(http_task: HttpTask, store: &'a Store, config: &'a config::Worker) -> Self {
        AdminTask {
            http_task,
            context: TaskContext { store, config },
        }
    }
}

impl<'a> SearchTask<'a> {
    pub fn new(http_task: HttpTask, store: &'a Store, config: &'a config::Worker) -> Self {
        let search_request =
            crate::request::search_request::SearchRequest::from_url(http_task.url());

        SearchTask {
            http_task,
            context: TaskContext { store, config },
            search_request,
        }
    }
}
