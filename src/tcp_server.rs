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

type RequestHandlerFn = Arc<dyn Fn(String) -> String + Send + Sync>;

pub struct TcpServer {
    ip: String,
    port: u16,

    request_handler: RequestHandlerFn,

    handle: Option<thread::JoinHandle<()>>,
    thread_pool: Arc<ThreadPool>,

    running: Arc<AtomicBool>,
}

impl TcpServer {
    pub fn new(
        ip: String,
        settings_http: &Http,
        rq_handler_obj: Arc<RequestHandler>,
    ) -> Result<Self, String> {
        let thread_pool = ThreadPool::new(settings_http.threads);
        let thread_pool = Arc::new(thread_pool);

        let request_handler: RequestHandlerFn = if let Some(destination) =
            settings_http.redirect.clone()
        {
            Arc::new(move |request: String| rq_handler_obj.redirect(request, destination.clone()))
        } else {
            Arc::new(move |request: String| rq_handler_obj.handle(request))
        };

        Ok(TcpServer {
            ip,
            port: settings_http.port,

            request_handler,

            handle: None,
            thread_pool,

            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start_thread(&mut self) {
        let ip = self.ip.clone();
        let port = self.port;

        let request_handler = self.request_handler.clone();
        let thread_pool = self.thread_pool.clone();

        self.running.store(true, Relaxed);
        let running = self.running.clone();

        println!("Starting TcpServer thread on {ip}:{port}");
        self.handle = Some(thread::spawn(move || {
            Self::run(ip, port, request_handler, thread_pool, running);
        }));
    }

    pub fn join_thread(&mut self) {
        if let Some(handle) = self.handle.take() {
            if let Err(_) = handle.join() {
                println!("Error joining TcpServer thread.");
            }
        } else {
            println!("No thread to join.");
            return;
        }
    }

    fn run(
        ip: String,
        port: u16,
        request_handler: RequestHandlerFn,
        thread_pool: Arc<ThreadPool>,
        running: Arc<AtomicBool>,
    ) {
        let listener = TcpListener::bind(format!("{}:{}", ip, port))
            .expect(format!("Failed to bind TcpListener to {ip}:{port}").as_str());
        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking TcpListener.");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let request_handler = request_handler.clone();
                    thread_pool.execute(Box::new(move || {
                        println!("TcpServer recieved new connection.");
                        Self::handle_client(stream, request_handler);
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

    fn handle_client(mut stream: TcpStream, request_handler: RequestHandlerFn) {
        let mut reader = BufReader::new(&mut stream);
        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        let request = String::from_utf8_lossy(&received).to_string();
        if !request.ends_with("\r\n\r\n") {
            return; // Internal Server Error or smthn
        }
        let response = request_handler(request);
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
