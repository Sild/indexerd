extern crate tiny_http;

use crate::config;
use crate::data::store::Store;

pub struct HttpTask {
    pub raw_req: tiny_http::Request,
}

impl HttpTask {
    pub fn new(req: tiny_http::Request) -> Self {
        HttpTask { raw_req: req }
    }

    pub fn respond(self, body: &str) {
        let resp = tiny_http::Response::from_string(body);
        _ = self.raw_req.respond(resp);
    }
}

pub struct TaskContext<'a> {
    pub store: &'a Store,
    pub config: &'a config::Worker,
}

pub struct WorkerTask<'a> {
    pub http_task: HttpTask,
    pub context: TaskContext<'a>,
}
