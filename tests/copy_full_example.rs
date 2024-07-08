use essentials::debug;
use futures_util::future::join_all;
use std::env;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::test]
async fn copy_full() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = mock_listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        let mut mock_file = tokio::fs::File::open("long_file.txt").await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx, None)
            .await
            .unwrap();
        left_tx.shutdown().await.unwrap();
        debug!("finished sendfile");
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
            tokio::spawn(async move {
                ::io::copy_tcp(&mut left_rx, &mut right_tx, None).await.unwrap();
                debug!("copied left -> right");
                right_tx.shutdown().await?;
                Ok::<(), std::io::Error>(())
            }),
            tokio::spawn(async move {
                ::io::copy_tcp(&mut right_rx, &mut left_tx, None).await?;
                debug!("copied right -> left");
                left_tx.shutdown().await?;
                Ok::<(), std::io::Error>(())
            }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    client.shutdown().await.unwrap();
    let mut len = 0;
    const EXPECTED: usize = 20 * 1024 * 1024;
    let mut buf = [0; 1024];
    loop {
        client.readable().await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        len += n;
    }
    // 20 MB file
    assert_eq!(len, EXPECTED);
}

#[tokio::test]
async fn copy_full_exact() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = mock_listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
        let mut mock_file = tokio::fs::File::open("long_file.txt").await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx, Some(20 * 1024 * 1024))
            .await
            .unwrap();
        left_tx.shutdown().await.unwrap();
        debug!("finished sendfile");
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
            tokio::spawn(async move {
                ::io::copy_tcp(&mut left_rx, &mut right_tx, None).await.unwrap();
                debug!("copied left -> right");
                right_tx.shutdown().await?;
                Ok::<(), std::io::Error>(())
            }),
            tokio::spawn(async move {
                ::io::copy_tcp(&mut right_rx, &mut left_tx, Some(20 * 1024 * 1024)).await?;
                debug!("copied right -> left");
                left_tx.shutdown().await?;
                Ok::<(), std::io::Error>(())
            }),
        ])
        .await;
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    client.shutdown().await.unwrap();
    let mut len = 0;
    const EXPECTED: usize = 20 * 1024 * 1024;
    let mut buf = [0; 1024];
    loop {
        client.readable().await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        //debug!("read {n} bytes, total {len} bytes");
        if n == 0 {
            break;
        }
        len += n;
    }
    // 20 MB file
    assert_eq!(len, EXPECTED);
}

#[tokio::test]
async fn copy_exact() {
    env::set_var("RUST_LOG", "debug");
    env::set_var("APP_ENV", "d");
    essentials::install();
    let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock_listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut left_rx, mut left_tx) = mock_listener.accept().await.unwrap().0.into_split();
        let mut buf = [0; 1024];
        let n = left_rx.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hell");
        let mut mock_file = tokio::fs::File::open("long_file.txt").await.unwrap();
        ::io::copy_file(&mut mock_file, &mut left_tx, Some(1024))
            .await
            .unwrap();
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
            tokio::spawn(
                async move { ::io::copy_tcp(&mut right_rx, &mut left_tx, Some(512)).await },
            ),
        ])
        .await
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap();
    });
    let mut client = tokio::net::TcpStream::connect(&addr).await.unwrap();
    client.write_all(b"hello").await.unwrap();
    let mut len = 0;
    let mut buf = [0; 1024];
    while len < 512 {
        client.readable().await.unwrap();
        let n = client.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        len += n;
    }
    assert_eq!(len, 512);
}
