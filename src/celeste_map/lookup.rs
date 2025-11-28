use std::{
    io,
    io::Read,
};

use allocator_api2::{
    alloc::{
        Allocator,
        Global,
    },
    vec::Vec,
};

use crate::celeste_map::string::{
    ReadStringError,
    SimpleString,
};

#[derive(Debug, Clone, derive_more::Deref)]
#[deref(forward)]
pub struct Lookup<A: Allocator = Global> {
    #[deref]
    inner: Vec<SimpleString<A>, A>,
}

impl<A: Allocator + Default> Default for Lookup<A> {
    fn default() -> Self {
        Self {
            inner: Vec::new_in(Default::default()),
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum LookupReadError {
    Io(io::Error),
    String(ReadStringError),
}

impl<A: Allocator + Clone> Lookup<A> {
    pub fn read_in<R: Read>(alloc: A, mut reader: R) -> Result<Self, LookupReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;

        let count: i16 = reader.read_prim()?;
        let mut list = Vec::with_capacity_in(count as _, alloc.clone());
        for _ in 0..count {
            let s = crate::celeste_map::string::read_dotnet_str(alloc.clone(), &mut reader)?;
            list.push(s);
        }
        Ok(Self { inner: list })
    }
}

impl<A: Allocator> Lookup<A> {
    pub(crate) fn read_indexed(&self, mut reader: impl Read) -> io::Result<&SimpleString<A>> {
        use dotnet_io_binary::io::prim::ReadPrim;

        let i: i16 = reader.read_prim()?;
        let a = &self.inner[i as usize];
        Ok(a)
    }
}
