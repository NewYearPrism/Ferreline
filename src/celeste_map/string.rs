use std::{
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
    io::Read,
    ops::Deref,
    str::Utf8Error,
};

use allocator_api2::{
    alloc::{
        Allocator,
        Global,
    },
    vec::Vec,
};
use dotnet_io_binary::io::string;

#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
pub enum ReadStringError {
    Read(string::ReadError),
    Utf8(Utf8Error),
}

pub(crate) fn read_dotnet_str<A: Allocator>(
    alloc: A,
    mut reader: impl Read,
) -> Result<SimpleString<A>, ReadStringError> {
    use dotnet_io_binary::io::string::ReadDotnetStr;

    let buf = reader.read_dotnet_str(|len| {
        let mut buf = Vec::with_capacity_in(len as _, alloc);
        buf.resize(len as _, 0);
        buf
    })?;
    Ok(buf.try_into()?)
}

#[derive(Clone, Debug)]
pub struct SimpleString<A: Allocator = Global>(Vec<u8, A>);

impl<A: Allocator> PartialEq for SimpleString<A> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A: Allocator> Eq for SimpleString<A> {}

impl<A: Allocator> PartialOrd for SimpleString<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Allocator> Ord for SimpleString<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<A: Allocator> Hash for SimpleString<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<A: Allocator> SimpleString<A> {
    pub(crate) fn new(s: Vec<u8, A>) -> Result<Self, Utf8Error> {
        str::from_utf8(&s)?;
        Ok(Self(s))
    }
}

impl<A: Allocator> AsRef<str> for SimpleString<A> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A: Allocator> Deref for SimpleString<A> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<A: Allocator> SimpleString<A> {
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.0).unwrap()
    }

    pub fn into_inner(self) -> Vec<u8, A> {
        self.0
    }
}

impl<A: Allocator> TryFrom<Vec<u8, A>> for SimpleString<A> {
    type Error = Utf8Error;

    fn try_from(t: Vec<u8, A>) -> Result<Self, Self::Error> {
        let a = Self::new(t)?;
        Ok(a)
    }
}

#[cfg(feature = "serde")]
impl<A: Allocator> serde::Serialize for SimpleString<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de, A: Allocator + Default> serde::Deserialize<'de> for SimpleString<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor<A>(core::marker::PhantomData<A>);
        impl<'de, A: Allocator + Default> serde::de::Visitor<'de> for StringVisitor<A> {
            type Value = SimpleString<A>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                let mut buf = Vec::with_capacity_in(v.len() as _, Default::default());
                buf.extend_from_slice(v.as_bytes());
                Ok(SimpleString(buf))
            }
        }
        deserializer.deserialize_str(StringVisitor(Default::default()))
    }
}
