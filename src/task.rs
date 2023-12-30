extern crate tiny_http;

use crate::config;
use crate::data::store::Store;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiny_http::Header;
use url::Url;

pub struct HttpTask {
    raw_req: tiny_http::Request,
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

fn parse_params(task: &HttpTask) -> Result<HashMap<String, String>> {
    let url = Url::parse((String::from("http://localhost:8088") + task.raw_req.url()).as_str())?;
    let mut params = HashMap::new();

    let pairs = match url.query() {
        Some(query) => query.split('&'),
        None => return Ok(params),
    };

    for pair in pairs {
        let mut split = pair.split('=');
        let key = split.next().unwrap_or("");
        let value = split.next().unwrap_or("");
        params.insert(key.to_string(), value.to_string());
    }

    Ok(params)
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SearchParams {
    name: Option<String>,
    id: Option<i32>,
    email: Option<String>,
}

impl SearchParams {
    fn try_from_bin(format: &str, data: &str) -> Result<Self> {
        let decoded = general_purpose::STANDARD_NO_PAD.decode(data)?;
        let decoded_str = std::str::from_utf8(&*decoded)?;

        {
            let pro = crate::proto::search_params::SearchParams {
                name: "t1".into(),
                id: 15,
                email: "52".into(),
            };
            let mut buf = vec![];
            pro.encode(&mut buf).unwrap();
            log::error!("{:?}", buf);
            let b64 = general_purpose::STANDARD_NO_PAD.encode(buf.clone());
            log::error!("{b64}");
        }

        match format {
            "json" => Ok(serde_json::from_str(decoded_str)?),
            "proto" => {
                let proto = crate::proto::search_params::SearchParams::decode(&*decoded)?;
                Ok(SearchParams {
                    name: proto.name.into(),
                    id: proto.id.into(),
                    email: proto.email.into(),
                })
            }
            _ => Ok(SearchParams::default()),
        }
    }
}

pub struct SearchTask<'a> {
    pub http_task: HttpTask,
    pub context: TaskContext<'a>,
    pub url_params: HashMap<String, String>, // url get-params
    pub search_params: SearchParams,         // parsed search params from &search_params=XXX
    pub is_malformed: bool,
    pub malformed_msg: String,
}

impl<'a> SearchTask<'a> {
    pub fn new(http_task: HttpTask, store: &'a Store, config: &'a config::Worker) -> Self {
        let url_params = match parse_params(&http_task) {
            Ok(params) => params,
            Err(e) => {
                log::error!("Failed to parse url params: {}", e);
                return SearchTask {
                    http_task,
                    context: TaskContext { store, config },
                    url_params: HashMap::default(),
                    search_params: SearchParams::default(),
                    is_malformed: true,
                    malformed_msg: format!("Failed to parse url params: {}", e),
                };
            }
        };
        log::debug!("url params: {:?}", url_params);
        let req_fmt = url_params
            .get("req_fmt")
            .unwrap_or(&String::from("proto"))
            .clone();

        let search_params = match req_fmt.as_str() {
            "proto" | "json" => SearchParams::try_from_bin(
                req_fmt.as_str(),
                url_params
                    .get("search_params")
                    .unwrap_or(&String::from(""))
                    .as_str(),
            ),
            _ => {
                return SearchTask {
                    http_task,
                    context: TaskContext { store, config },
                    url_params,
                    search_params: SearchParams::default(),
                    is_malformed: true,
                    malformed_msg: format!("Unknown req_fmt={}", req_fmt),
                }
            }
        };

        match search_params {
            Ok(search_params) => SearchTask {
                http_task,
                context: TaskContext { store, config },
                url_params,
                search_params,
                is_malformed: false,
                malformed_msg: String::default(),
            },
            Err(e) => {
                return SearchTask {
                    http_task,
                    context: TaskContext { store, config },
                    url_params,
                    search_params: SearchParams::default(),
                    is_malformed: true,
                    malformed_msg: e.to_string() + ":" + e.backtrace().to_string().as_str(),
                }
            }
        }
    }
}
