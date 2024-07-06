mod file;
mod tcp;

use tokio::{
    fs::File,
    io,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

/// Copy data from a read half to a write half.
/// This function is only available on linux platforms and uses splice.
pub async fn copy_tcp<'a>(
    r: &'a mut OwnedReadHalf,
    w: &'a mut OwnedWriteHalf,
    length: Option<usize>,
) -> io::Result<usize> {
    if let Some(length) = length {
        tcp::copy_exact(r, w, length).await
    } else {
        tcp::copy(r, w).await
    }
}
/// Copy data from a file to a write half.
/// This function is only available on linux platforms and uses sendfile.
pub async fn copy_file<'a>(
    r: &'a mut File,
    w: &'a mut OwnedWriteHalf,
    length: Option<usize>,
) -> io::Result<usize> {
    if let Some(length) = length {
        file::copy_exact(r, w, length).await
    } else {
        file::copy(r, w).await
    }
}
