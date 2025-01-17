use serde::Deserialize;
use serde_json::Result;
use std::collections::HashSet;
use std::io::Read;

#[derive(Default, Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Server {
    pub service: Service,
    pub engine: Engine,
    pub updater: Updater,
}

#[derive(Default, Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Service {
    pub listening_port: u16,
}

#[derive(Default, Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Updater {
    pub db: DB,
    pub swap_interval: u64,
}

#[derive(Default, Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct DB {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db_name: String,
}

#[derive(Default, Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Engine {
    pub worker: Worker,
    pub non_worker_cores: HashSet<u16>,
}

#[derive(Default, Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct Worker {
    pub need_multi: bool,
}

impl Server {
    pub fn from_file(path: &str) -> Result<Server> {
        let mut file = std::fs::File::open(path).expect("file should open read only");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let conf: Server = serde_json::from_str(file_content.as_str())?;
        Ok(conf)
    }
}
