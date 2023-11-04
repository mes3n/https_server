use std::sync::Arc;

use my_server::{
    ipc_listener::IpcListener, request_handler::RequestHandler, settings::Settings,
    tcp_server::TcpServer, tls_server::TlsServer,
};

fn main() {
    let settings = Settings::new("Settings.toml");
    let settings = Arc::new(settings);

    let request_handler = RequestHandler::new(settings.server.document_root.clone());
    let request_handler = Arc::new(request_handler);

    let mut tcp_server = TcpServer::new(
        settings.server.ip.clone(),
        &settings.http,
        request_handler.clone(),
    );
    tcp_server.start_thread();

    let mut tls_server = TlsServer::new(
        settings.server.ip.clone(),
        &settings.https,
        request_handler.clone(),
    );
    tls_server.start_thread();

    IpcListener::new().listen_block();

    println!("Stopping Server...");
}
