#[allow(dead_code)]
mod support;

use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use support::*;

integration_test!(serve_smoke_test, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("README.md"), b"# Some content");
    let binary = area.binary();
    let path = area.path.to_path_buf();

    let (sender1, receiver1) = channel::<()>();
    let (sender2, receiver2) = channel::<()>();

    std::thread::spawn(move || {
        let mut handle = Command::new(binary)
            .args(&["serve"])
            .current_dir(path)
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("Unable to spawn command");

        sender2.send(()).unwrap();
        receiver1.recv().unwrap();
        handle.kill().unwrap();
        sender2.send(()).unwrap();
    });

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Make a request to the locally running server
    use std::io::Read;
    use std::io::Write;
    use std::net::TcpStream;

    receiver2.recv().unwrap();

    let mut stream = TcpStream::connect("localhost:4001").unwrap();

    let mut request_data = String::new();
    request_data.push_str("GET / HTTP/1.0");
    request_data.push_str("\r\n");
    request_data.push_str("Host: localhost");
    request_data.push_str("\r\n");
    request_data.push_str("Connection: close"); // <== Here!
    request_data.push_str("\r\n");
    request_data.push_str("\r\n");

    stream.write_all(request_data.as_bytes()).unwrap();

    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();

    sender1.send(()).unwrap();
    receiver2.recv().unwrap();

    assert!(buf.contains("Some content"));
});
