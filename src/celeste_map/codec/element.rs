use std::{
    borrow::BorrowMut,
    collections::BTreeMap,
    io::Read,
};

use dotnet_io_binary::io::prim;

use super::{
    attribute::{
        Attribute,
        AttributeValue,
    },
    lookup::Lookup,
    string::SimpleString,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Element {
    pub name: SimpleString,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<SimpleString, AttributeValue>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub children: Vec<Element>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("unable to read length attribute list or child list")]
    Length(#[from] prim::ReadError),
    #[error("unable to get lookup data")]
    Lookup(#[from] super::lookup::ReadIndexGetError),
    #[error("unable to read attribute")]
    Attribute(#[from] super::attribute::ReadError),
}

impl Element {
    pub(crate) fn read<R: Read>(
        mut reader: impl Read + BorrowMut<R>,
        lookup: &Lookup,
    ) -> Result<Self, ReadError>
    where
        for<'r> &'r mut R: BorrowMut<R>,
    {
        use dotnet_io_binary::io::prim::ReadPrim;

        let name = lookup.read_index_get(reader.borrow_mut())?.clone();
        let attr_count: u8 = reader.read_prim()?;
        let mut attributes = BTreeMap::new();
        for _ in 0..attr_count {
            let attr = Attribute::read(reader.borrow_mut(), lookup)?;
            attributes.insert(attr.name, attr.value);
        }
        let child_count: u16 = reader.read_prim()?;
        let mut children = Vec::with_capacity(child_count as _);
        for _ in 0..child_count {
            let elem = Element::read(reader.borrow_mut(), lookup)?;
            children.push(elem);
        }
        Ok(Self {
            name,
            attributes,
            children,
        })
    }
}
