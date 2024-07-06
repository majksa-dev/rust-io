use std::env;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::test]
async fn copy_file() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        let mut mock_file = tokio::fs::File::open("long_file.txt").await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx, None)
            .await
            .unwrap();
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
async fn copy_file_exact() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        let mut mock_file = tokio::fs::File::open("long_file.txt").await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx, Some(1024 * 1024))
            .await
            .unwrap();
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
    assert_eq!(len, 1024 * 1024);
}
