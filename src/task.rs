extern crate tiny_http;

use crate::data::store::Store;
use crate::{config, data};
use std::collections::HashMap;
use tiny_http::Header;
use url::Url;

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

impl<'a> AdminTask<'a> {
    pub fn new(http_task: HttpTask, store: &'a Store, config: &'a config::Worker) -> Self {
        AdminTask {
            http_task,
            context: TaskContext { store, config },
        }
    }
}

fn parse_params(task: &HttpTask) -> HashMap<String, String> {
    let url = Url::parse(task.raw_req.url()).unwrap();
    let mut params = HashMap::new();

    let pairs = match url.query() {
        Some(query) => query.split('&'),
        None => return params,
    };

    for pair in pairs {
        let mut split = pair.split('=');
        let key = split.next().unwrap_or("");
        let value = split.next().unwrap_or("");
        params.insert(key.to_string(), value.to_string());
    }

    params
}

#[derive(Default)]
pub struct SearchParams {
    name: String,
    id: i32,
    email: String,
}

pub struct SearchTask<'a> {
    pub http_task: HttpTask,
    pub context: TaskContext<'a>,
    pub search_params: SearchParams,
    pub is_malformed: bool,
}

impl<'a> SearchTask<'a> {
    pub fn new(http_task: HttpTask, store: &'a Store, config: &'a config::Worker) -> Self {
        let url_params = parse_params(&http_task);
        let mut malformed = false;
        let default_req_type = String::from("plain");
        let req_type = url_params.get("req_type").unwrap_or(&default_req_type);

        let search_params = match req_type.as_str() {
            "proto" => SearchParams::default(),
            "plain" => SearchParams::default(),
            _ => {
                malformed = true;
                SearchParams::default()
            }
        };
        SearchTask {
            http_task,
            context: TaskContext { store, config },
            search_params,
            is_malformed: malformed,
        }
    }
}
