use std::io::Read;

use dotnet_io_binary::io::{
    prim,
    prim::ReadPrim,
};

use super::string::SimpleString;

#[derive(Debug, Clone, Default, derive_more::Deref)]
#[deref(forward)]
pub struct Lookup(Vec<SimpleString>);

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("unable to read length")]
    Length(#[from] prim::ReadError),
    #[error("unable to read string")]
    String(#[from] super::string::ReadError),
}

impl Lookup {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, ReadError> {
        let count: i16 = reader.read_prim()?;
        let mut list = Vec::with_capacity(count as _);
        for _ in 0..count {
            let s = super::string::read_dotnet_str(&mut reader)?;
            list.push(s);
        }
        Ok(Self(list))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReadIndexGetError {
    #[error("unable to get lookup data")]
    ReadIndex(#[from] prim::ReadError),
    #[error("index `{0}` out of bounds (for {1} lookup items)")]
    OutOfBounds(i16, usize),
}

impl Lookup {
    pub(crate) fn read_index_get(
        &self,
        mut reader: impl Read,
    ) -> Result<&SimpleString, ReadIndexGetError> {
        let i: i16 = reader.read_prim()?;
        let a = self
            .get(i as usize)
            .ok_or(ReadIndexGetError::OutOfBounds(i, self.len()))?;
        Ok(a)
    }
}
