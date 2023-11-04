use std::{
    io::{stdin, stdout, Write},
    os::unix::net::UnixStream,
};

use my_server::ipc_listener::{IpcListener, SOCKET_PATH};

fn main() {
    let mut command = String::new();

    let mut stream = match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error connecting to socket at {SOCKET_PATH}: {}", e);
            return;
        }
    };

    loop {
        command.clear();
        print!(">>> ");
        stdout().flush().ok();

        stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        match command {
            "exit" => break,
            "help" => {
                println!("Commands:");
                println!("exit - exit the client");
                println!("help - print this help message");
                println!("stop - stop the server");
                continue;
            }
            _ => {}
        }

        stream.write_all(command.as_bytes()).unwrap();
        println!("{}", IpcListener::read_stream(&mut stream));

        if command == "stop" {
            break;
        }
    }
}
