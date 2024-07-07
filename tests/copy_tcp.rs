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
        let chunk = [b'x'; 1024].as_slice();
        for _ in 0..(20*1024) {
            server.write_all(chunk).await.unwrap();
        }
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
            tokio::spawn(async move { ::io::copy_tcp(&mut left_rx, &mut right_tx, None).await }),
            tokio::spawn(async move { ::io::copy_tcp(&mut right_rx, &mut left_tx, None).await }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    let mut len = 0;
    let mut buf = [0; 1024];
    loop {
        let n = client.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        len += n;
    }
    // 20 MB file
    assert_eq!(len, 20 * 1024 * 1024);
}

#[tokio::test]
async fn copy_tcp_long() {
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
        server.write_all(b"").await.unwrap();
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
            tokio::spawn(async move { ::io::copy_tcp(&mut left_rx, &mut right_tx, None).await }),
            tokio::spawn(async move { ::io::copy_tcp(&mut right_rx, &mut left_tx, None).await }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    let mut buf = [0; 1024];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"hi");
}

#[tokio::test]
async fn copy_tcp_exact() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut server, _) = mock_listener.accept().await.unwrap();
        let mut buf = [0; 1024];
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hell");
        server.write_all(b"hihihihihihi").await.unwrap();
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
            tokio::spawn(async move { ::io::copy_tcp(&mut left_rx, &mut right_tx, Some(4)).await }),
            tokio::spawn(async move { ::io::copy_tcp(&mut right_rx, &mut left_tx, Some(4)).await }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    let mut buf = [0; 1024];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"hihi");
}
