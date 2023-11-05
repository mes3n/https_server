use serde::Deserialize;
use std::fs::read_to_string;

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

impl Settings {
    pub fn new(from: &str) -> Self {
        match toml::from_str(
            &read_to_string(from).expect(&format!("File {from} could not be found in working directory")),
        ) {
            Ok(settings) => settings,
            Err(e) => panic!("Failed to parse settings in {from}: {e}"),
        }
    }
}
