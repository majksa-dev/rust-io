use std::env;

use assert_fs::fixture::{FileWriteStr, PathChild};
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
    let mock_file = assert_fs::TempDir::new().unwrap().child("mock_file");
    mock_file.write_str("hi").unwrap();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = mock_listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        let mut mock_file = tokio::fs::File::open(mock_file.path()).await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx).await.unwrap();
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