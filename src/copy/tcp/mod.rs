#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::copy;

#[cfg(not(target_os = "linux"))]
use tokio::{
    io,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

/// Copy data from a read half to a write half.
/// This function is only available on non-linux platforms and uses tokio::io::copy.
#[cfg(not(target_os = "linux"))]
pub async fn copy<'a>(r: &'a mut OwnedReadHalf, w: &'a mut OwnedWriteHalf) -> io::Result<()> {
    use essentials::debug;

    debug!("copying tcp stream using tokio::io::copy");
    io::copy(r, w).await?;
    Ok(())
}
