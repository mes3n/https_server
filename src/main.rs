use std::sync::Arc;

use my_server::{
    ipc_listener::IpcListener, request_handler::RequestHandler, settings::Settings,
    tcp_server::TcpServer, tls_server::TlsServer,
};

fn main() {
    let settings = match Settings::new("Settings.toml") {
        Ok(settings) => settings,
        Err(err) => {
            println!("Error creating Settings: {err:?}");
            return;
        }
    };
    let settings = Arc::new(settings);

    let request_handler = match RequestHandler::new(settings.server.document_root.clone()) {
        Ok(request_handler) => request_handler,
        Err(err) => {
            println!("Error creating Request Handler: {err:?}");
            return;
        }
    };
    let request_handler = Arc::new(request_handler);

    let _tcp_server = match TcpServer::new(
        settings.server.ip.clone(),
        &settings.http,
        request_handler.clone(),
    ) {
        Ok(mut tcp_server) => {
            tcp_server.start_thread();
            Some(tcp_server)
        }
        Err(err) => {
            println!("Error creating TcpServer: {:?}", err);
            None
        }
    };

    let _tls_server = match TlsServer::new(
        settings.server.ip.clone(),
        &settings.https,
        request_handler.clone(),
    ) {
        Ok(mut tls_server) => {
            tls_server.start_thread();
            Some(tls_server)
        }
        Err(err) => {
            println!("Error creating TlsServer: {:?}", err);
            None
        }
    };

    IpcListener::new().listen_block();

    println!("Stopping Server...");
}
