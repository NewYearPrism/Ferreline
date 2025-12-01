use std::{
    io,
    io::Read,
};

pub const HEADER: &[u8] = b"\x0bCELESTE MAP";

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("IO error happened on reading header")]
    Io(#[from] io::Error),
    #[error("the header does NOT matches \"CELESTE MAP\"")]
    NotMatch,
}

pub(crate) fn read_header(mut reader: impl Read) -> Result<(), HeaderError> {
    let mut buf = [0u8; HEADER.len()];
    reader.read_exact(&mut buf)?;
    if HEADER != buf {
        Err(HeaderError::NotMatch)?
    }
    Ok(())
}
