use essentials::debug;
use libc::off_t;
use std::future::Future;
use std::io::Result;
use std::os::{fd::RawFd, unix::prelude::AsRawFd};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::tcp::OwnedWriteHalf,
};

pub const MAX_LENGTH: usize = off_t::max_value() as usize;
pub const MAX_CHUNK: usize = 0x7ffff000; // according to the Linux docs, 0x7ffff000 is the maximum length for one sendfile()

pub struct SendFile {
    r: RawFd,
    w: RawFd,
    offset: usize,
    remaining: usize,
    copied: usize,
}

impl SendFile {
    fn raw_send_file(&mut self) -> Result<usize> {
        match sendfile_n(
            self.r,
            self.w,
            &mut self.offset,
            MAX_CHUNK.min(self.remaining),
        ) {
            -1 => Err(io::Error::last_os_error()),
            n => {
                let n = n as usize;
                self.copied += n;
                self.remaining -= n;
                Ok(n)
            }
        }
    }
}

// Impl async trait
impl Future for SendFile {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.raw_send_file() {
                Ok(0) => break Poll::Ready(Ok(self.copied as usize)),
                Ok(_) => continue, // Attempt to write some more bytes.
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    cx.waker().wake_by_ref();
                    break Poll::Pending;
                }
                Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue, // Try again.
                Err(err) => break Poll::Ready(Err(err)),
            }
        }
    }
}

/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy<'a>(r: &'a mut File, w: &'a mut OwnedWriteHalf) -> io::Result<usize> {
    copy_exact(r, w, r.metadata().await?.len() as usize).await
}

/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy_exact<'a>(
    r: &'a mut File,
    w: &'a mut OwnedWriteHalf,
    length: usize,
) -> io::Result<usize> {
    debug!("copying file to tcp stream using sendfile");
    if length == 0 {
        return Ok(0);
    };
    let mut buffered_length = 0;
    let sendfile_length = if length > MAX_LENGTH {
        buffered_length = length - MAX_LENGTH; // bytes that exceed sendfile()'s capacity
        MAX_LENGTH
    } else {
        length
    };
    let rfd = r.as_raw_fd();
    let wfd = w.as_ref().as_raw_fd();
    let mut n = SendFile {
        r: rfd,
        w: wfd,
        offset: 0,
        remaining: sendfile_length,
        copied: 0,
    }
    .await?;
    if buffered_length > 0 {
        n += tokio::io::copy(&mut r.take(length as u64), w).await? as usize;
    }
    Ok(n)
}

fn sendfile_n(r: i32, w: i32, offset: &mut usize, n: usize) -> isize {
    let inner_offset = *offset as i64;
    let result = unsafe { libc::sendfile(w, r, inner_offset as *mut i64, n) };
    *offset = inner_offset as usize;
    result
}

fn is_wouldblock() -> bool {
    use libc::{EAGAIN, EWOULDBLOCK};
    let errno = unsafe { *libc::__errno_location() };
    errno == EWOULDBLOCK || errno == EAGAIN
}
