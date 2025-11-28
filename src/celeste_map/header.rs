use std::io::Read;

pub const HEADER: &[u8] = b"\x0bCELESTE MAP";

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum HeaderReadError {
    Io(std::io::Error),
    NotMatch,
}

pub(crate) fn read_header(mut reader: impl Read) -> Result<(), HeaderReadError> {
    let mut buf = [0u8; HEADER.len()];
    reader.read_exact(&mut buf)?;
    if HEADER != buf {
        Err(HeaderReadError::NotMatch)?
    }
    Ok(())
}
