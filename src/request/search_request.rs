use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use strum_macros::Display;
use strum_macros::EnumString;
use url::Url;

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchParams {
    name: String,
    id: i32,
    email: String,
}

#[derive(Debug, Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
enum RequestFormat {
    Json,
    Proto,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SearchRequest {
    #[allow(dead_code)]
    req_fmt: RequestFormat, // used only for debug logging
    pub search_params: SearchParams,
}

impl SearchRequest {
    pub fn from_url(url: &str) -> Result<Self> {
        let get_params = Self::parse_get_params(url)?;

        let req_fmt = match get_params.get("req_fmt") {
            Some(req_fmt) => RequestFormat::from_str(req_fmt)?,
            None => RequestFormat::Json,
        };

        let search_params = match get_params.get("search_params") {
            Some(params) => SearchParams::try_from_bin(&req_fmt, params)?,
            None => return Err(anyhow::anyhow!("search_params not found in url={}", url)),
        };

        let search_params = SearchRequest {
            req_fmt,
            search_params,
        };

        Ok(search_params)
    }

    fn parse_get_params(url: &str) -> Result<HashMap<String, String>> {
        let url = Url::parse((String::from("http://localhost:8088") + url).as_str())?;
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
}

impl SearchParams {
    fn try_from_bin(req_fmt: &RequestFormat, data: &str) -> Result<Self> {
        let decoded = general_purpose::STANDARD_NO_PAD.decode(data)?;
        let decoded_str = std::str::from_utf8(&*decoded)?;

        match req_fmt {
            RequestFormat::Json => Ok(serde_json::from_str(decoded_str)?),
            RequestFormat::Proto => {
                let proto = crate::proto::search_params::SearchParams::decode(&*decoded)?;
                Ok(SearchParams {
                    name: proto.name,
                    id: proto.id,
                    email: proto.email,
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn proto_to_base64<T: prost::Message>(proto: &T) -> String {
        let mut buf = vec![];
        proto.encode(&mut buf).unwrap();
        log::error!("{:?}", buf);
        general_purpose::STANDARD_NO_PAD.encode(buf.clone())
    }

    fn obj_to_base64<T: serde::Serialize>(obj: &T) -> String {
        let json = serde_json::to_string(obj).unwrap();
        general_purpose::STANDARD_NO_PAD.encode(json)
    }

    #[test]
    fn test_search_request_parse_get_params() {
        let given = SearchRequest::parse_get_params("/some_path?key1=val1&key2=val2").unwrap();
        let expected = HashMap::from([
            ("key1".to_string(), "val1".to_string()),
            ("key2".to_string(), "val2".to_string()),
        ]);
        assert_eq!(expected, given);
    }

    #[test]
    fn test_search_request_from_url_json() {
        let sp = SearchParams {
            name: "t2".into(),
            id: 16,
            email: "53".into(),
        };
        let search_params = obj_to_base64(&sp);

        let expected = SearchRequest {
            req_fmt: RequestFormat::Json,
            search_params: sp,
        };

        for req_fmt in ["req_fmt=json", ""] {
            let given = SearchRequest::from_url(
                format!("/some_path?{req_fmt}&search_params={search_params}").as_str(),
            )
            .unwrap();
            assert_eq!(expected, given);
        }
    }

    #[test]
    fn test_search_request_from_url_proto() {
        let sp = SearchParams {
            name: "t3".into(),
            id: 17,
            email: "54".into(),
        };

        let proto = crate::proto::search_params::SearchParams {
            name: sp.name.clone().into(),
            id: sp.id.clone().into(),
            email: sp.email.clone().into(),
        };

        let search_params = proto_to_base64(&proto);

        let expected = SearchRequest {
            req_fmt: RequestFormat::Proto,
            search_params: sp,
        };

        let given = SearchRequest::from_url(
            format!("/some_path?req_fmt=proto&&search_params={search_params}").as_str(),
        )
        .unwrap();

        assert_eq!(expected, given);
    }
}
