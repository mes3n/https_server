use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread,
};

use crate::{request_handler::RequestHandler, settings::Http, thread_pool::ThreadPool};

pub struct TcpServer {
    ip: String,
    port: u16,

    destination: Option<String>,
    request_handler: Arc<RequestHandler>,

    handle: Option<thread::JoinHandle<()>>,
    thread_pool: Arc<ThreadPool>,

    running: Arc<AtomicBool>,
}

impl TcpServer {
    pub fn new(ip: String, settings_http: &Http, request_handler: Arc<RequestHandler>) -> Self {
        let thread_pool = ThreadPool::new(settings_http.threads);
        let thread_pool = Arc::new(thread_pool);

        let destination = settings_http.redirect.clone();
        TcpServer {
            ip,
            port: settings_http.port,

            destination,
            request_handler,

            handle: None,
            thread_pool,

            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_thread(&mut self) {
        let destination = self.destination.clone();
        if destination.is_none() {
            println!("No destination set for TcpServer, not starting thread");
            return;
        }

        let ip = self.ip.clone();
        let port = self.port;
        let destination = destination.unwrap();

        let request_handler = self.request_handler.clone();
        let thread_pool = self.thread_pool.clone();

        self.running.store(true, Relaxed);
        let running = self.running.clone();

        println!("Starting TcpServer thread on {ip}:{port} with redirect to {destination}");
        self.handle = Some(thread::spawn(move || {
            Self::run(ip, port, destination, request_handler, thread_pool, running);
        }));
    }

    pub fn join_thread(&mut self) {
        if self.handle.is_none() {
            println!("No thread to join.");
            return;
        }

        if let Some(handle) = self.handle.take() {
            handle.join().expect("Failed to join thread.");
        }
    }

    fn run(
        ip: String,
        port: u16,
        destination: String,
        request_handler: Arc<RequestHandler>,
        thread_pool: Arc<ThreadPool>,
        running: Arc<AtomicBool>,
    ) {
        let listener = TcpListener::bind(format!("{}:{}", ip, port)).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking TcpListener.");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let request_handler = request_handler.clone();
                    let destination = destination.clone();
                    thread_pool.execute(Box::new(move || {
                        println!("TcpServer recieved new connection.");
                        Self::handle_client(stream, request_handler, destination);
                    }));
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => {}
                    e => {
                        println!("Caught Error in TcpServer: {}. Not Handeled.", e);
                    }
                },
            }
            thread::sleep(std::time::Duration::from_millis(80));
            if !running.load(Relaxed) {
                break;
            }
        }

        println!("TcpServer thread exited cleanly.");
    }

    fn handle_client(
        mut stream: TcpStream,
        request_handler: Arc<RequestHandler>,
        destination: String,
    ) {
        let mut reader = BufReader::new(&mut stream);
        let received = reader.fill_buf().unwrap().to_vec();
        println!("Received: {}", String::from_utf8_lossy(&received));
        let request = String::from_utf8_lossy(&received).to_string();
        if !request.ends_with("\r\n\r\n") {
            return;  // Internal Server Error or smthn
        }
        reader.consume(received.len());
        let response = request_handler.redirect(request, destination);
        stream.write_all(response.as_bytes()).unwrap();
        println!("Sent response.");
    }
}

impl Drop for TcpServer {
    fn drop(&mut self) {
        println!("Sending terminate message to TcpServer.");
        self.running.store(false, Relaxed);
        self.join_thread();
        println!("TcpServer shut down.");
    }
}
