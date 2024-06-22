use std::os::unix::prelude::AsRawFd;
use std::ptr;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::tcp::OwnedWriteHalf,
};

/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy<'a>(r: &'a mut File, w: &'a mut OwnedWriteHalf) -> io::Result<usize> {
    use essentials::debug;

    debug!("copying file to tcp stream using sendfile");
    // create pipe
    let rfd = r.as_raw_fd();
    let wfd = w.as_ref().as_raw_fd();
    let mut n: usize = 0;
    loop {
        w.as_ref().writable().await?;
        match unsafe { libc::sendfile(rfd, wfd, ptr::null_mut(), usize::MAX) } {
            x if x > 0 => n += x as usize,
            0 => {
                break;
            }
            x if x < 0 && is_wouldblock() => continue,
            _ => return Err(io::Error::last_os_error()),
        }
        // clear readiness (EPOLLIN)
        let _ = r.read(&mut [0u8; 0]).await;
    }
    Ok(n)
}

fn is_wouldblock() -> bool {
    use libc::{EAGAIN, EWOULDBLOCK};
    let errno = unsafe { *libc::__errno_location() };
    errno == EWOULDBLOCK || errno == EAGAIN
}
