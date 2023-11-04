use std::{collections::BTreeMap, fs::read_to_string, path::Path};

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
    document_root: String,
}

impl RequestHandler {
    pub fn new(document_root: String) -> Self {
        RequestHandler {
            http_codes: HttpCodes::new(),
            document_root,
        }
    }

    pub fn handle(&self, request: String) -> String {
        if request.starts_with("GET") {
            let path = request.splitn(3, " ").nth(1).unwrap();
            let path = format!("{}/{}", self.document_root, path);
            let path = Path::new(&path);

            return if path.is_file() {
                self.format_response(200, path.to_str().unwrap().to_string())
            } else if path.is_dir() {
                let path = path.join("index.html");
                if path.exists() {
                    self.format_response(200, path.to_str().unwrap().to_string())
                } else {
                    self.format_response(404, self.get_path("404.html"))
                }
            } else {
                self.format_response(404, self.get_path("404.html"))
            };
        }
        self.format_response(400, self.get_path("400.html"))
    }

    pub fn redirect(&self, request: String, destination: String) -> String {
        if request.starts_with("GET /") {
            let path = request.splitn(3, " ").nth(1).unwrap();
            format!(
                "{}\r\nLocation: {destination}{path}\r\n\r\n",
                self.http_codes.get(301)
            )
        } else {
            self.format_response(400, self.get_path("400.html"))
        }
    }

    fn get_path(&self, file: &str) -> String {
        let mut path = self.document_root.clone().to_owned();
        path.push_str(file);
        path
    }

    fn format_response(&self, code: u16, path: String) -> String {
        let content = read_to_string(path).unwrap();
        format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            self.http_codes.get(code),
            content.len(),
            content
        )
    }
}
