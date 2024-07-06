#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::copy;

#[cfg(target_os = "linux")]
pub use linux::copy_exact;

#[cfg(not(target_os = "linux"))]
use tokio::{
    io,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

/// Copy data from a read half to a write half.
/// This function is only available on non-linux platforms and uses tokio::io::copy.
#[cfg(not(target_os = "linux"))]
pub async fn copy<'a>(r: &'a mut OwnedReadHalf, w: &'a mut OwnedWriteHalf) -> io::Result<usize> {
    use essentials::debug;

    debug!("copying tcp stream using tokio::io::copy");
    Ok(io::copy(r, w).await? as usize)
}

/// Copy data from a read half to a write half.
/// This function is only available on linux platforms and uses splice.
#[cfg(not(target_os = "linux"))]
pub async fn copy_exact<'a>(
    r: &'a mut OwnedReadHalf,
    w: &'a mut OwnedWriteHalf,
    length: usize,
) -> io::Result<usize> {
    use essentials::debug;
    use tokio::io::AsyncReadExt;

    debug!("copying tcp stream using tokio::io::copy");
    Ok(io::copy(&mut r.take(length as u64), w).await? as usize)
}
