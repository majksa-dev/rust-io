#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::copy_tcp;

#[cfg(not(target_os = "linux"))]
use tokio::{
    io,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

#[cfg(not(target_os = "linux"))]
pub async fn copy<'a>(rfd: &'a mut OwnedReadHalf, wfd: &'a mut OwnedWriteHalf) -> io::Result<()> {
    io::copy(rfd, wfd).await?;
    Ok(())
}
