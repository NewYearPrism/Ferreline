use std::{
    io::Read,
    ops::Deref,
    str::Utf8Error,
};

use dotnet_io_binary::io::string;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("unable to read the byte string")]
    Content(#[from] string::ReadError),
    #[error("the bytes are not valid UTF-8")]
    Utf8(#[from] Utf8Error),
}

pub(crate) fn read_dotnet_str(mut reader: impl Read) -> Result<SimpleString, ReadError> {
    use dotnet_io_binary::io::string::ReadDotnetStr;

    let buf = reader.read_dotnet_str(|len| vec![0; len as _])?;
    Ok(buf.try_into()?)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleString(Vec<u8>);

impl SimpleString {
    pub(crate) fn new(s: Vec<u8>) -> Result<Self, Utf8Error> {
        str::from_utf8(&s)?;
        Ok(Self(s))
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for SimpleString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl SimpleString {
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.0).unwrap()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<Vec<u8>> for SimpleString {
    type Error = Utf8Error;

    fn try_from(t: Vec<u8>) -> Result<Self, Self::Error> {
        let a = Self::new(t)?;
        Ok(a)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SimpleString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SimpleString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor;
        impl<'de> serde::de::Visitor<'de> for StringVisitor {
            type Value = SimpleString;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                let mut buf = Vec::with_capacity(v.len() as _);
                buf.extend_from_slice(v.as_bytes());
                Ok(SimpleString(buf))
            }
        }
        deserializer.deserialize_str(StringVisitor)
    }
}
