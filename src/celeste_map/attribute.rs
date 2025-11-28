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

use crate::celeste_map::{
    lookup::Lookup,
    rle::Rle,
    string::{
        ReadStringError,
        SimpleString,
    },
};

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum AttributeReadError {
    Io(io::Error),
    UnknownType(derive_more::TryFromReprError<u8>),
    String(ReadStringError),
    /// when get RLE attribute type but length is not multiple of 2.
    Rle,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u8)]
enum AttributeType {
    Boolean = 0,
    Byte,
    Int16,
    Int32,
    Single,
    Lookup,
    Str,
    Rle,
}

impl AttributeType {
    fn read(mut reader: impl Read) -> Result<Self, AttributeReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;
        let a: u8 = reader.read_prim()?;
        let t = a.try_into()?;
        Ok(t)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "A: Default", serialize = "")),
    serde(tag = "type", content = "value")
)]
pub enum AttributeValue<A: Allocator = Global> {
    Boolean(bool),
    Byte(u8),
    Short(i16),
    Integer(i32),
    Float(f32),
    Lookup(SimpleString<A>),
    String(SimpleString<A>),
    RunLengthEncoded(Rle<A>),
}

impl AttributeType {
    fn read_value<A: Allocator + Clone>(
        self,
        alloc: A,
        mut reader: impl Read,
        lookup: &Lookup<A>,
    ) -> Result<AttributeValue<A>, AttributeReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;
        let attr = match self {
            AttributeType::Boolean => {
                let p: u8 = reader.read_prim()?;
                AttributeValue::Boolean(p != 0)
            }
            AttributeType::Byte => AttributeValue::Byte(reader.read_prim()?),
            AttributeType::Int16 => AttributeValue::Short(reader.read_prim()?),
            AttributeType::Int32 => AttributeValue::Integer(reader.read_prim()?),
            AttributeType::Single => AttributeValue::Float(reader.read_prim()?),
            AttributeType::Lookup => {
                let lk = lookup.read_indexed(reader)?;
                AttributeValue::Lookup(lk.clone())
            }
            AttributeType::Str => {
                let buf = crate::celeste_map::string::read_dotnet_str(alloc, &mut reader)?;
                AttributeValue::String(buf)
            }
            AttributeType::Rle => {
                let len: i16 = reader.read_prim()?;
                let mut buf = {
                    let mut buf = Vec::with_capacity_in(len as _, alloc);
                    buf.resize(len as _, 0);
                    buf
                };
                reader.read_exact(&mut buf)?;
                AttributeValue::RunLengthEncoded(Rle::new(buf).ok_or(AttributeReadError::Rle)?)
            }
        };
        Ok(attr)
    }
}

impl<A: Allocator + Clone> AttributeValue<A> {
    fn read(
        alloc: A,
        mut reader: impl Read,
        lookup: &Lookup<A>,
    ) -> Result<Self, AttributeReadError> {
        let t = AttributeType::read(&mut reader)?;
        let a = t.read_value(alloc, reader, lookup)?;
        Ok(a)
    }
}

#[derive(Debug, Clone)]
pub struct Attribute<A: Allocator = Global> {
    pub name: SimpleString<A>,
    pub value: AttributeValue<A>,
}

impl<A: Allocator + Clone> Attribute<A> {
    pub(crate) fn read(
        alloc: A,
        mut reader: impl Read,
        lookup: &Lookup<A>,
    ) -> Result<Self, AttributeReadError> {
        let name = lookup.read_indexed(&mut reader)?.clone();
        let value = AttributeValue::read(alloc, reader, lookup)?;
        Ok(Self { name, value })
    }
}
