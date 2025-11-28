use std::io::Read;

use allocator_api2::alloc::{
    Allocator,
    Global,
};

use crate::{
    celeste_map::{
        element::Element,
        lookup::Lookup,
    },
    string::SimpleString,
};

pub mod attribute;
pub mod element;
pub mod header;
pub mod lookup;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "A: Default + Clone", serialize = ""))
)]
pub struct CelesteMap<A: Allocator = Global> {
    #[cfg_attr(feature = "serde", serde(rename = "PackageName"))]
    pub package_name: SimpleString<A>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lookup: Lookup<A>,
    #[cfg_attr(feature = "serde", serde(rename = "Map"))]
    pub tree: Element<A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum CelesteMapReadError {
    Header(header::HeaderReadError),
    PackageName(crate::string::ReadStringError),
    Lookup(lookup::LookupReadError),
    Element(element::ElementReadError),
}

impl CelesteMap {
    pub fn read(reader: impl Read) -> Result<Self, CelesteMapReadError> {
        Self::read_in(Default::default(), reader)
    }
}

impl<A: Allocator + Clone> CelesteMap<A> {
    pub fn read_in(alloc: A, mut reader: impl Read) -> Result<Self, CelesteMapReadError> {
        header::read_header(&mut reader)?;
        let package_name = crate::string::read_dotnet_str(alloc.clone(), &mut reader)?;
        let lookup = Lookup::read_in(alloc.clone(), &mut reader)?;
        let tree = Element::read(alloc, reader, &lookup)?;
        Ok(Self {
            package_name,
            lookup,
            tree,
        })
    }
}
