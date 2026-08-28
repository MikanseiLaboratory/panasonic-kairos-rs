#![cfg(feature = "std")]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;

use panasonic_kairos::simple::SOURCE_A;
use panasonic_kairos::tcp::Client;
use panasonic_kairos::TcpConfig;

#[test]
fn blocking_force_source_sends_companion_command() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        let mut stream = BufReader::new(socket);
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();
        stream.get_mut().write_all(b"OK\r\n").unwrap();
        stream.get_mut().flush().unwrap();
        line.trim_end().to_string()
    });
    let mut client = Client::connect(
        TcpConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout_ms(2_000),
    )
    .unwrap();
    client
        .force_source("Main", "Background", SOURCE_A, "IP1")
        .unwrap();
    assert_eq!(
        handle.join().unwrap(),
        "SCENES.Main.Layers.Background.sourceA=IP1"
    );
}
