mod zero_copy;

use essentials::debug;
use tokio::net::tcp::OwnedReadHalf;
use tokio::{io, net::tcp::OwnedWriteHalf};
use zero_copy::zero_copy_unidirectional;

/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy<'a>(r: &'a mut OwnedReadHalf, w: &'a mut OwnedWriteHalf) -> io::Result<usize> {
    debug!("copying tcp stream using splice");
    Ok(zero_copy_unidirectional(r, w, None).await? as usize)
}

/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy_exact<'a>(
    r: &'a mut OwnedReadHalf,
    w: &'a mut OwnedWriteHalf,
    length: usize,
) -> io::Result<usize> {
    debug!("copying tcp stream using splice");
    if length == 0 {
        return Ok(0);
    };
    debug!("copying tcp stream using splice");
    Ok(zero_copy_unidirectional(r, w, Some(length as u64)).await? as usize)
}
