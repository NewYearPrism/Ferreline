use std::io::Read;

use dotnet_io_binary::io::{
    prim,
    prim::ReadPrim,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum AttributeType {
    Boolean = 0,
    Byte,
    Int16,
    Int32,
    Single,
    Lookup,
    Str,
    Rle,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("unable to read attribute type")]
    Read(#[from] prim::ReadError),
    #[error("invalid attribute type")]
    Invalid(#[from] derive_more::TryFromReprError<u8>),
}

impl AttributeType {
    pub(crate) fn read(mut reader: impl Read) -> Result<Self, ReadError> {
        let a: u8 = reader.read_prim()?;
        let t = a.try_into()?;
        Ok(t)
    }
}
