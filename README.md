# HTTPS Webserver

A webserver with a customizable configuration file for serving html files or redirection requests. 
Built using Rust with native-tls openssl for HTTPS requests.

## Prerequisites

### Dependencies

The server uses native-tls and as such a native implementation for tls encryption. 
The openssl package is recommended.

### Certificate

A PFX certificate is needed to run the https server. If it is just to be used for teseting, a self-signed certifacate can be generated with:

```shell
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365
openssl pkcs12 -inkey key.pem -in cert.pem -export -out identity.pfx
```

### Compilation

Install cargo with preferred method:
```shell
apt install cargo
pacman -S cargo
```

Compile source code, preferably with cargo:
```shell
cargo build 
```

#### Client

If cargo is installed the IPC communication clinet can be built with:
```shell
cargo build --bin client
```

## Configuration

Configuration can be changed from the Settings.toml file. The webserver needs to be restarted
for the changes to take effect.

| **Setting** | Values
| --- | --- |
| *server.ip* | The ip address the server will be hosted on |
| *server.document_root* | A path to the root directory for the servers html files |
| *https.port* | The port for the https server |
| *https.redirect* | An url for the https server to redirect to |
| *https.thread* | Amount of threads available to the https server |
| *https.ssl.indentity* | pfx file used for https certification |
| *https.ssl.password* | Password for the pfx file |
| *http.port* | The port for the http server |
| *http.redirect* | An url for the http server to redirect to |
| *http.thread* | Amount of threads available to the http server |

## IPC Interface

The server creates a socket file in /tmp which can be used for IPC through the client application provided in src/bin/client.rs. This file can be compiled and run through
```shell
cargo run --bin client
```

The implementation of commands has so far been limited to a ```stop``` command to stop the server along with ```help``` and ```exit``` for manging the client itself.

