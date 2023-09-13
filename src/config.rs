use serde::Deserialize;
use serde_json::Result;
use std::io::Read;
#[derive(Debug, Deserialize)]
pub struct DBConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db_name: String,
}

impl DBConfig {
    pub fn from_file(path: &str) -> Result<DBConfig> {
        let mut file = std::fs::File::open(path).expect("file should open read only");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        let conf: DBConfig = serde_json::from_str(file_content.as_str())?;
        Ok(conf)
    }
}
