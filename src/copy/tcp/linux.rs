use std::os::unix::prelude::AsRawFd;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
};

const BUFFERSIZE: usize = if cfg!(not(target_os = "linux")) {
    0x4000 // 16k read/write buffer
} else {
    0x10000 // 64k pipe buffer
};

/// Copy data from a read half to a write half.
/// This function is only available on linux platforms and uses splice.
pub async fn copy<'a>(r: &'a mut OwnedReadHalf, w: &'a mut OwnedWriteHalf) -> io::Result<()> {
    use essentials::debug;

    debug!("copying tcp stream using splice");
    // create pipe
    let mut pipes = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
    let (rpipe, wpipe) = unsafe {
        if libc::pipe2(pipes.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to call pipe"));
        }
        (pipes.assume_init()[0], pipes.assume_init()[1])
    };
    // get raw fd
    let rfd = r.as_ref().as_raw_fd();
    let wfd = w.as_ref().as_raw_fd();
    let mut n: usize = 0;
    let mut done = false;

    'LOOP: loop {
        // read until the socket buffer is empty
        // or the pipe is filled
        r.as_ref().readable().await?;
        while n < BUFFERSIZE {
            match splice_n(rfd, wpipe, BUFFERSIZE - n) {
                x if x > 0 => n += x as usize,
                x if x == 0 => {
                    done = true;
                    break;
                }
                x if x < 0 && is_wouldblock() => break,
                _ => break 'LOOP,
            }
        }
        // write until the pipe is empty
        while n > 0 {
            w.as_ref().writable().await?;
            match splice_n(rpipe, wfd, n) {
                x if x > 0 => n -= x as usize,
                x if x < 0 && is_wouldblock() => {
                    // clear readiness (EPOLLOUT)
                    let _ = r.as_ref().try_write(&[0u8; 0]);
                }
                _ => break 'LOOP,
            }
        }
        // complete
        if done {
            break;
        }
        // clear readiness (EPOLLIN)
        let _ = r.as_ref().try_read(&mut [0u8; 0]);
    }

    unsafe {
        libc::close(rpipe);
        libc::close(wpipe);
    }
    Ok(())
}

fn splice_n(r: i32, w: i32, n: usize) -> isize {
    use libc::{loff_t, SPLICE_F_MOVE, SPLICE_F_NONBLOCK};
    unsafe {
        libc::splice(
            r,
            0 as *mut loff_t,
            w,
            0 as *mut loff_t,
            n,
            SPLICE_F_MOVE | SPLICE_F_NONBLOCK,
        )
    }
}

fn is_wouldblock() -> bool {
    use libc::{EAGAIN, EWOULDBLOCK};
    let errno = unsafe { *libc::__errno_location() };
    errno == EWOULDBLOCK || errno == EAGAIN
}
