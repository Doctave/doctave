use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bus::{Bus, BusReader};
use crossbeam_channel::Receiver;
use tungstenite::error::Error as TungsteniteError;
use tungstenite::handshake::HandshakeError;
use tungstenite::protocol::WebSocket;

/// Sets up a websocket server listening for livereload connections,
/// and pushes updates to the browser when notified.
///
/// Uses post 35729, as per the livereload.js standard.
///
/// Note that this server does not serve the actual livereload.js payload.
/// This module expects the client to already have access to it by some
/// other means.
pub struct LivereloadServer {
    channel: Receiver<()>,
    bus: Arc<Mutex<Bus<()>>>,
}

impl LivereloadServer {
    pub fn new(channel: Receiver<()>) -> Self {
        LivereloadServer {
            channel,
            bus: Arc::new(Mutex::new(Bus::new(128))),
        }
    }

    /// Consumes the server and runs until the program terminates. Sets up
    /// thread to listen for connections, and another for broadcasting
    /// updates to them.
    pub fn run(self) {
        let bus_clone = self.bus.clone();
        thread::Builder::new()
            .name("livereload-listener".into())
            .spawn(move || run_listener(bus_clone))
            .unwrap();

        for _msg in self.channel {
            self.bus.lock().unwrap().broadcast(());
        }
    }
}

fn run_listener(bus: Arc<Mutex<Bus<()>>>) {
    let server = std::net::TcpListener::bind("127.0.0.1:35729").unwrap();

    for stream in server.incoming().filter_map(Result::ok) {
        let receiver = bus.lock().unwrap().add_rx();

        thread::Builder::new()
            .name("livereload-connection".into())
            .spawn(move || {
                handle_websocket(stream, receiver);
            })
            .unwrap();
    }
}

fn handle_websocket(stream: std::net::TcpStream, mut listener: BusReader<()>) {
    let result = || -> io::Result<()> {
        let mut websocket = tungstenite::accept(stream).map_err(|err| match err {
            HandshakeError::Failure(e) => map_tungstenite_error(e),
            other => io::Error::new(io::ErrorKind::Other, other),
        })?;

        if livereload_handshake(&mut websocket).is_err() {
            // If the handshake fails, bail. Just can happen for
            // example when the user spams the reload button in
            // the browser.
            return Ok(());
        }

        loop {
            if let Ok(_msg) = listener.recv_timeout(Duration::from_millis(1000)) {
                websocket
                    .write_message(
                        r#"
                        {
                            "command": "reload",
                            "path": "",
                            "liveCSS": true
                        }
                        "#
                        .into(),
                    )
                    .map_err(|e| map_tungstenite_error(e))?;
            } else {
                websocket
                    .write_message(tungstenite::Message::Ping(Vec::new()))
                    .map_err(|e| map_tungstenite_error(e))?;
            }
        }
    };

    match (result)() {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
        // Unexpected errors that are not just disconnects.
        Err(e) => println!(
            "Livereload client disconnected due to an unexpected error: {}.",
            e
        ),
    };
}

fn livereload_handshake(websocket: &mut WebSocket<std::net::TcpStream>) -> io::Result<()> {
    let msg = websocket.read_message().map_err(map_tungstenite_error)?;

    if msg.is_text() {
        let parsed: serde_json::Value = serde_json::from_str(msg.to_text().unwrap())?;

        if parsed["command"] != "hello" {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid handshake"));
        }

        let response = r#"
        {
            "command": "hello",
            "protocols": ["http://livereload.com/protocols/official-7"],
            "serverName": "doctave"
        }
        "#;

        websocket
            .write_message(response.into())
            .map_err(map_tungstenite_error)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Invalid handshake"))
    }
}

fn map_tungstenite_error(error: TungsteniteError) -> io::Error {
    match error {
        TungsteniteError::Io(io_error) => io_error,
        e => io::Error::new(io::ErrorKind::Other, e),
    }
}
