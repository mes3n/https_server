use native_tls::{Identity, TlsAcceptor, TlsStream};

use std::{
    fs::read,
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread,
};

use crate::{request_handler::RequestHandler, settings::Https, thread_pool::ThreadPool};

type RequestHandlerFn = Arc<dyn Fn(String) -> String + Send + Sync>;

pub struct TlsServer {
    ip: String,
    port: u16,

    acceptor: Arc<TlsAcceptor>,
    request_handler: RequestHandlerFn,

    handle: Option<thread::JoinHandle<()>>,
    thread_pool: Arc<ThreadPool>,

    running: Arc<AtomicBool>,
}

impl TlsServer {
    pub fn new(
        ip: String,
        settings_https: &Https,
        rq_handler_obj: Arc<RequestHandler>,
    ) -> Result<Self, String> {
        let identity = match Identity::from_pkcs12(
            match &read(&settings_https.ssl.identity) {
                Ok(identity) => identity,
                Err(e) => {
                    return Result::Err(format!(
                        "Could not read identity file {} for TlsServer: {e}",
                        &settings_https.ssl.identity
                    ));
                }
            },
            &settings_https.ssl.password,
        ) {
            Ok(identity) => identity,
            Err(e) => {
                return Result::Err(format!(
                    "Could not create identity for TlsServer. Got error: {e}"
                ));
            }
        };

        let acceptor = match TlsAcceptor::new(identity) {
            Ok(acceptor) => acceptor,
            Err(e) => {
                return Result::Err(format!(
                    "Could not create TlsAcceptor for TlsServer. Got error: {e}"
                ));
            }
        };
        let acceptor = Arc::new(acceptor);

        let thread_pool = ThreadPool::new(settings_https.threads);
        let thread_pool = Arc::new(thread_pool);

        let request_handler: RequestHandlerFn = if let Some(destination) =
            settings_https.redirect.clone()
        {
            Arc::new(move |request: String| rq_handler_obj.redirect(request, destination.clone()))
        } else {
            Arc::new(move |request: String| rq_handler_obj.handle(request))
        };

        Ok(TlsServer {
            ip,
            port: settings_https.port,

            acceptor,
            request_handler,

            handle: None,
            thread_pool,

            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start_thread(&mut self) {
        let ip = self.ip.clone();
        let port = self.port;

        let acceptor = self.acceptor.clone();
        let request_handler = self.request_handler.clone();

        let thread_pool = self.thread_pool.clone();

        self.running.store(true, Relaxed);
        let running = self.running.clone();

        println!("Starting TlsServer thread on {ip}:{port}.");
        self.handle = Some(thread::spawn(move || {
            Self::run(ip, port, acceptor, request_handler, thread_pool, running);
        }));
    }

    pub fn join_thread(&mut self) {
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Err(e) => println!("Error joining TlsServer thread. {e:?}"),
                _ => {}
            };
        }
    }

    fn run(
        ip: String,
        port: u16,
        acceptor: Arc<TlsAcceptor>,
        request_handler: RequestHandlerFn,
        thread_pool: Arc<ThreadPool>,
        running: Arc<AtomicBool>,
    ) {
        let listener = TcpListener::bind(format!("{}:{}", ip, port))
            .expect(format!("Failed to bind TlsListener to {ip}:{port}").as_str());
        listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking TlsListener.");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let acceptor = acceptor.clone();
                    let request_handler = request_handler.clone();
                    thread_pool.execute(Box::new(move || {
                        let stream = match acceptor.accept(stream) {
                            Ok(stream) => stream,
                            Err(e) => {
                                println!("Handshake error: {}. Not Handeled.", e);
                                return;
                            }
                        };
                        println!(
                            "TlsServer recieved new connection.",
                        );
                        Self::handle_client(stream, request_handler);
                    }));
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => {}
                    e => {
                        println!("Caught Error in TlsServer: {}. Not Handeled.", e);
                    }
                },
            }
            thread::sleep(std::time::Duration::from_millis(80));
            if !running.load(Relaxed) {
                break;
            }
        }

        println!("TlsServer thread exited cleanly.");
    }

    fn handle_client(mut stream: TlsStream<TcpStream>, request_handler: RequestHandlerFn) {
        let mut reader = BufReader::new(&mut stream);
        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        let request = String::from_utf8_lossy(&received).to_string();
        if !request.ends_with("\r\n\r\n") {
            return; // Internal Server Error or smthn
        }
        let response = request_handler(request);
        match stream.write_all(response.as_bytes()) {
            Ok(_) => println!("Sent response."),
            Err(e) => println!("Failed to send response. {e:?}")
        };
    }
}

impl Drop for TlsServer {
    fn drop(&mut self) {
        println!("Sending terminate message to TlsServer.");
        self.running.store(false, Relaxed);
        self.join_thread();
        println!("TlsServer shut down.");
    }
}
