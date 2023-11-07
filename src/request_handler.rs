use std::{collections::BTreeMap, error::Error, fs::read_to_string, path::PathBuf};

#[derive(Clone)]
pub struct HttpCodes {
    codes: BTreeMap<u16, &'static str>,
}

impl HttpCodes {
    pub fn new() -> Self {
        let mut codes = BTreeMap::new();
        codes.insert(200, "HTTP/1.1 200 OK");
        codes.insert(301, "HTTP/1.1 301 Moved Permanently");
        codes.insert(400, "HTTP/1.1 400 Bad Request");
        codes.insert(404, "HTTP/1.1 404 Not Found");
        codes.insert(501, "HTTP/1.1 501 Not Implemented");
        HttpCodes { codes }
    }

    pub fn get(&self, code: u16) -> &str {
        match self.codes.get(&code) {
            Some(text) => text,
            None => "Unknown",
        }
    }
}

#[derive(Clone)]
pub struct RequestHandler {
    http_codes: HttpCodes,
    document_root: PathBuf,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

impl RequestHandler {
    pub fn new(document_root: String) -> Result<Self> {
        let path = PathBuf::from(&document_root);
        if !path.is_dir() {
            return Err(format!("No such directory. {document_root} not found.").into());
        }

        Ok(RequestHandler {
            http_codes: HttpCodes::new(),
            document_root: path,
        })
    }

    pub fn handle(&self, request: String) -> String {
        if request.starts_with("GET") {
            let path = match request.splitn(3, " ").nth(1) {
                Some(item) => {
                    let item = item.replace("/", "");
                    self.get_path(&item)
                }
                _ => {
                    return self.format_response(400, self.get_path("400.html"))
                } 
            };

            if path.is_file() {
                return self.format_response(200, path);
            } else if path.is_dir() {
                let path = path.join("index.html");
                if path.exists() {
                    return self.format_response(200, path);
                }
            }
            self.format_response(404, self.get_path("404.html"))
        } else {
            self.format_response(501, self.get_path("501.html"))
        }
    }

    pub fn redirect(&self, request: String, destination: String) -> String {
        if request.starts_with("GET") {
            let path = request.splitn(3, " ").nth(1).unwrap();
            format!(
                "{}\r\nLocation: {destination}{path}\r\n\r\n",
                self.http_codes.get(301)
            )
        } else {
            self.format_response(400, self.get_path("400.html"))
        }
    }

    fn get_path(&self, file: &str) -> PathBuf {
        self.document_root.join(file)
    }

    fn format_response(&self, code: u16, path: PathBuf) -> String {
        let content = read_to_string(path).unwrap_or("".to_owned());
        format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            self.http_codes.get(code),
            content.len(),
            content
        )
    }
}
