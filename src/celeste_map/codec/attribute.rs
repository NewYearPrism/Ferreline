pub mod types;

use std::{
    io,
    io::Read,
};

use dotnet_io_binary::io::{
    prim,
    prim::ReadPrim,
};
use types::AttributeType;

use super::{
    lookup::Lookup,
    rle::Rle,
    string::SimpleString,
};

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("unable to read attribute type")]
    Type(#[from] types::ReadError),
    #[error("unable to read attribute name")]
    Name(#[from] super::lookup::ReadIndexGetError),
    #[error("unable to read attribute of type `{attr_type:?}`")]
    Content {
        attr_type: AttributeType,
        #[source]
        source: ReadErrorInner,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ReadErrorInner {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Length(#[from] prim::ReadError),
    #[error(transparent)]
    Lookup(#[from] super::lookup::ReadIndexGetError),
    #[error(transparent)]
    String(#[from] super::string::ReadError),
    #[error("RLE length is not multiple of 2")]
    Rle,
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(bound(serialize = "")),
    serde(rename_all = "snake_case"),
    serde(untagged)
)]
pub enum AttributeValue {
    Boolean(bool),
    Byte(u8),
    Short(i16),
    Integer(i32),
    Float(f32),
    Lookup(SimpleString),
    String(SimpleString),
    RunLengthEncoded(Rle),
}

impl AttributeValue {
    fn read(mut reader: impl Read, lookup: &Lookup) -> Result<Self, ReadError> {
        let t = AttributeType::read(&mut reader)?;
        let attr = match t {
            AttributeType::Boolean => {
                let p: u8 = reader.read_prim().map_err(|source| ReadError::Content {
                    attr_type: t,
                    source: source.into(),
                })?;
                AttributeValue::Boolean(p != 0)
            }
            AttributeType::Byte => {
                AttributeValue::Byte(reader.read_prim().map_err(|source| ReadError::Content {
                    attr_type: t,
                    source: source.into(),
                })?)
            }
            AttributeType::Int16 => {
                AttributeValue::Short(reader.read_prim().map_err(|source| ReadError::Content {
                    attr_type: t,
                    source: source.into(),
                })?)
            }
            AttributeType::Int32 => {
                AttributeValue::Integer(reader.read_prim().map_err(|source| {
                    ReadError::Content {
                        attr_type: t,
                        source: source.into(),
                    }
                })?)
            }
            AttributeType::Single => {
                AttributeValue::Float(reader.read_prim().map_err(|source| ReadError::Content {
                    attr_type: t,
                    source: source.into(),
                })?)
            }
            AttributeType::Lookup => {
                let lk = lookup
                    .read_index_get(reader)
                    .map_err(|source| ReadError::Content {
                        attr_type: t,
                        source: source.into(),
                    })?;
                AttributeValue::Lookup(lk.clone())
            }
            AttributeType::Str => {
                let buf = super::string::read_dotnet_str(&mut reader).map_err(|source| {
                    ReadError::Content {
                        attr_type: t,
                        source: source.into(),
                    }
                })?;
                AttributeValue::String(buf)
            }
            AttributeType::Rle => {
                let len: i16 = reader.read_prim().map_err(|source| ReadError::Content {
                    attr_type: t,
                    source: source.into(),
                })?;
                let mut buf = vec![0; len as _];
                reader
                    .read_exact(&mut buf)
                    .map_err(|source| ReadError::Content {
                        attr_type: t,
                        source: source.into(),
                    })?;
                AttributeValue::RunLengthEncoded(Rle::new(buf).ok_or(ReadError::Content {
                    attr_type: t,
                    source: ReadErrorInner::Rle,
                })?)
            }
        };
        Ok(attr)
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: SimpleString,
    pub value: AttributeValue,
}

impl Attribute {
    pub(crate) fn read(mut reader: impl Read, lookup: &Lookup) -> Result<Self, ReadError> {
        let name = lookup.read_index_get(&mut reader)?.clone();
        let value = AttributeValue::read(reader, lookup)?;
        Ok(Self { name, value })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AttributeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde_untagged::UntaggedEnumVisitor::new()
            .bool(|b| Ok(Self::Boolean(b)))
            .i64(|i| Ok(Self::Integer(i as _)))
            .f64(|f| Ok(Self::Float(f as _)))
            .string(|s| {
                let mut buf = Vec::with_capacity(s.len());
                buf.extend_from_slice(s.as_bytes());
                Ok(Self::String(buf.try_into().unwrap()))
            })
            .map(|m| m.deserialize().map(Self::RunLengthEncoded))
            .deserialize(deserializer)
    }
}
