#![cfg(feature = "tokio")]

use panasonic_kairos::simple::SOURCE_A;
use panasonic_kairos::tcp_async::Client;
use panasonic_kairos::TcpConfig;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

async fn spawn_ok_server() -> (u16, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut stream = BufReader::new(socket);
        let mut line = String::new();
        stream.read_line(&mut line).await.unwrap();
        stream.write_all(b"OK\r\n").await.unwrap();
        stream.flush().await.unwrap();
        line.trim_end().to_string()
    });
    (port, handle)
}

#[tokio::test]
async fn force_source_sends_companion_command() {
    let (port, handle) = spawn_ok_server().await;
    let mut client = Client::connect(
        TcpConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout_ms(2_000),
    )
    .await
    .unwrap();
    client
        .force_source("Main", "Background", SOURCE_A, "IP1")
        .await
        .unwrap();
    let sent = handle.await.unwrap();
    assert_eq!(sent, "SCENES.Main.Layers.Background.sourceA=IP1");
}

#[tokio::test]
async fn list_reads_until_blank_line() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut stream = BufReader::new(socket);
        let mut line = String::new();
        stream.read_line(&mut line).await.unwrap();
        stream
            .write_all(b"SCENES.Main\r\nSCENES.Templates\r\n\r\n")
            .await
            .unwrap();
        stream.flush().await.unwrap();
        line.trim_end().to_string()
    });
    let mut client = Client::connect(
        TcpConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout_ms(2_000),
    )
    .await
    .unwrap();
    let rows = client.list("SCENES").await.unwrap();
    assert_eq!(server.await.unwrap(), "list:SCENES");
    assert_eq!(rows, vec!["SCENES.Main", "SCENES.Templates"]);
}

#[tokio::test]
async fn keep_alive_is_empty_line() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 2];
        tokio::io::AsyncReadExt::read_exact(&mut socket, &mut buf)
            .await
            .unwrap();
        buf
    });
    let mut client = Client::connect(
        TcpConfig::new("127.0.0.1")
            .with_port(port)
            .with_timeout_ms(2_000),
    )
    .await
    .unwrap();
    client.keep_alive().await.unwrap();
    assert_eq!(&server.await.unwrap(), b"\r\n");
}
