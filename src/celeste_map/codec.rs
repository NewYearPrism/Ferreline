use std::io::Read;

use element::Element;
use lookup::Lookup;
use string::SimpleString;

pub mod attribute;
pub mod element;
pub mod header;
pub mod lookup;
pub mod rle;
pub mod string;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CelesteMap {
    #[cfg_attr(feature = "serde", serde(rename = "PackageName"))]
    pub package_name: SimpleString,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lookup: Lookup,
    #[cfg_attr(feature = "serde", serde(rename = "Map"))]
    pub tree: Element,
}

#[derive(Debug, thiserror::Error)]
pub enum CelesteMapReadError {
    #[error("data source does not have a valid header")]
    Header(#[from] header::HeaderError),
    #[error("unable to read package name")]
    PackageName(#[from] string::ReadError),
    #[error("unable to read lookup table")]
    Lookup(#[from] lookup::ReadError),
    #[error("unable to read map tree")]
    Element(#[from] element::ReadError),
}

impl CelesteMap {
    pub fn read(mut reader: impl Read) -> Result<Self, CelesteMapReadError> {
        header::read_header(&mut reader)?;
        let package_name = string::read_dotnet_str(&mut reader)?;
        let lookup = Lookup::read(&mut reader)?;
        let tree = Element::read(reader, &lookup)?;
        Ok(Self {
            package_name,
            lookup,
            tree,
        })
    }
}
