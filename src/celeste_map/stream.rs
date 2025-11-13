use std::{
    io,
    io::Read,
    ops::{
        ControlFlow,
        DerefMut,
    },
};

use dotnet_io_binary::io::{
    rw_prim::ReadPrim,
    rw_string,
    rw_string::ReadDotnetStr,
};

pub fn decode(r: impl Read) -> HeaderValidate<impl Read> {
    HeaderValidate(r)
}

pub struct HeaderValidate<R>(R);

impl<R: Read> HeaderValidate<R> {
    pub fn validate(
        mut self,
        buf: &mut Vec<u8>,
    ) -> Result<Option<PackageName<R>>, rw_string::ReadError> {
        self.0.read_dotnet_str_to(buf)?;
        if buf.as_slice() != super::HEADER.as_bytes() {
            return Ok(None);
        }
        Ok(Some(PackageName(self.0)))
    }
}

pub struct PackageName<R>(R);

impl<R: Read> PackageName<R> {
    pub fn read(
        mut self,
        buf: &mut Vec<u8>,
    ) -> Result<(&[u8], LookupTable<R>), rw_string::ReadError> {
        self.0.read_dotnet_str_to(buf)?;
        Ok((buf, LookupTable(self.0)))
    }
}

pub struct LookupTable<R>(R);

impl<R: Read> LookupTable<R> {
    pub fn start<B: DerefMut<Target = [u8]>>(
        mut self,
        buf_fn: impl FnMut(u32) -> B,
    ) -> Result<Lookup<R, impl FnMut(u32) -> B>, io::Error> {
        let len = self.0.read_prim()?;
        Ok(Lookup {
            len,
            reader: self.0,
            buf_fn,
            left: len,
        })
    }
}

pub struct Lookup<R, F> {
    len: i16,
    reader: R,
    buf_fn: F,
    left: i16,
}

impl<R, F> Lookup<R, F> {
    pub fn len(&self) -> i16 {
        self.len
    }
}

impl<R: Read, F: FnMut(u32) -> B, B: DerefMut<Target = [u8]>> iteratail::Iteratail
    for Lookup<R, F>
{
    type Item = Result<B, rw_string::ReadError>;
    type Tail = ();

    fn next_or_tail(mut self) -> ControlFlow<Self::Tail, (Self, Self::Item)> {
        if self.left == 0 {
            ControlFlow::Break(())?
        }
        let a = self.reader.read_dotnet_str(|l| (self.buf_fn)(l));
        if a.is_ok() {
            self.left -= 1;
        }
        ControlFlow::Continue((self, a))
    }
}
