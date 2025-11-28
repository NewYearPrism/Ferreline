use std::{
    borrow::BorrowMut,
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
use hashbrown::{
    DefaultHashBuilder,
    HashMap,
};

use crate::celeste_map::{
    attribute::{
        Attribute,
        AttributeReadError,
        AttributeValue,
    },
    lookup::Lookup,
    string::SimpleString,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "A: Default + Clone", serialize = ""))
)]
pub struct Element<A: Allocator = Global> {
    pub name: SimpleString<A>,
    pub attributes: HashMap<SimpleString<A>, AttributeValue<A>, DefaultHashBuilder, A>,
    pub children: Vec<Element<A>, A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum ElementReadError {
    Io(io::Error),
    Attribute(AttributeReadError),
}

impl<A: Allocator + Clone> Element<A> {
    pub(crate) fn read<R: Read>(
        alloc: A,
        mut reader: impl Read + BorrowMut<R>,
        lookup: &Lookup<A>,
    ) -> Result<Self, ElementReadError>
    where
        for<'r> &'r mut R: BorrowMut<R>,
    {
        use dotnet_io_binary::io::prim::ReadPrim;

        let name = lookup.read_indexed(reader.borrow_mut())?.clone();
        let attr_count: u8 = reader.read_prim()?;
        let mut attributes = HashMap::with_capacity_in(attr_count as _, alloc.clone());
        for _ in 0..attr_count {
            let attr = Attribute::read(alloc.clone(), reader.borrow_mut(), lookup)?;
            attributes.insert(attr.name, attr.value);
        }
        let child_count: u16 = reader.read_prim()?;
        let mut children = Vec::with_capacity_in(child_count as _, alloc.clone());
        for _ in 0..child_count {
            let elem = Element::read(alloc.clone(), reader.borrow_mut(), lookup)?;
            children.push(elem);
        }
        Ok(Self {
            name,
            attributes,
            children,
        })
    }
}
