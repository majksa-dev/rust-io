#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::copy_tcp;

#[cfg(not(target_os = "linux"))]
use tokio::{fs::File, io, net::tcp::OwnedWriteHalf};

/// Copy data from a file to a write half.
/// This function is only available on non-linux platforms and uses tokio::io::copy.
#[cfg(not(target_os = "linux"))]
pub async fn copy<'a>(r: &'a mut File, w: &'a mut OwnedWriteHalf) -> io::Result<()> {
    use essentials::debug;

    debug!("copying file to tcp stream using tokio::io::copy");
    io::copy(r, w).await?;
    Ok(())
}
