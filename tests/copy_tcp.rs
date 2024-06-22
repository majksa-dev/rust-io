use std::env;

use futures_util::future::join_all;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::test]
async fn copy_tcp() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut server, _) = mock_listener.accept().await.unwrap();
        let mut buf = [0; 1024];
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        server.write_all(b"hi").await.unwrap();
    });
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = listener.accept().await.unwrap().0.into_split();
        let (mut right_rx, mut right_tx) = tokio::net::TcpStream::connect(&mock_addr)
            .await
            .unwrap()
            .into_split();
        join_all([
            tokio::spawn(async move { ::io::copy_tcp(&mut left_rx, &mut right_tx).await }),
            tokio::spawn(async move { ::io::copy_tcp(&mut right_rx, &mut left_tx).await }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    let mut buf = [0; 1024];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"hi");
}
