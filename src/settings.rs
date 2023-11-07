use serde::Deserialize;
use std::{error::Error, fs::read_to_string};

#[derive(Deserialize)]
pub struct Settings {
    pub server: Server,
    pub https: Https,
    pub http: Http,
}

#[derive(Deserialize)]
pub struct Server {
    pub ip: String,
    pub domain: Option<String>,
    pub document_root: String,
}

#[derive(Deserialize)]
pub struct Https {
    pub port: u16,
    pub redirect: Option<String>,
    #[serde(default = "threads")]
    pub threads: usize,
    pub ssl: SSL,
}

#[derive(Deserialize)]
pub struct SSL {
    pub identity: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Http {
    pub port: u16,
    pub redirect: Option<String>,
    #[serde(default = "threads")]
    pub threads: usize,
}

fn threads() -> usize {
    4
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

impl Settings {
    pub fn new(path: &str) -> Result<Self> {
        match toml::from_str(&match read_to_string(path) {
            Ok(content) => content,
            Err(err) => return Err(err.into()),
        }) {
            Ok(settings) => Ok(settings),
            Err(err) => return Err(err.into()),
        }
    }
}
