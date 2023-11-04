use ctrlc::set_handler;
use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

pub const SOCKET_PATH: &str = "/tmp/my_server.sock";

pub mod ipc_commands {
    pub const STOP: &str = "stop";
}

pub struct IpcListener {
    listener: UnixListener,
}

impl IpcListener {
    pub fn new() -> IpcListener {
        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(listener) => listener,
            Err(e) => {
                println!("Error creating IpcListener from file {SOCKET_PATH}: {e}");
                exit(1);
            }
        };
        IpcListener { listener }
    }

    pub fn listen_block(&self) {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        set_handler(move || stop_clone.store(true, Relaxed))
            .expect("Error setting Ctrl-C handler.");

        self.listener
            .set_nonblocking(true)
            .expect("Failed to set nonblocking unix listener.");
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => loop {
                    let message = IpcListener::read_stream(&stream);
                    println!("IPC listener recieved: {message}");
                    match message.trim() {
                        ipc_commands::STOP => {
                            stream
                                .write_all(b"Stopping blocking IpcListener.")
                                .expect("Error writing to stream.");
                            return;
                        }
                        _ => {
                            stream
                                .write_all(b"Unknown command.")
                                .expect("Error writing to stream.");
                        }
                    }
                },
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => {}
                    _ => {
                        println!("Error: {}", e);
                    }
                },
            }
            sleep(Duration::from_millis(80));
            if stop.load(Relaxed) {
                println!("IPC listener recieved termination signal. Stopping.");
                break;
            }
        }
    }

    pub fn read_stream(mut stream: &UnixStream) -> String {
        let mut reader = BufReader::new(&mut stream);
        let received = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());

        String::from_utf8_lossy(&received)
            .trim_matches(char::from(0))
            .to_owned()
    }
}

impl Drop for IpcListener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
        println!("Dropping IpcListener");
    }
}
