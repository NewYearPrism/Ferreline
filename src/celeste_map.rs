use crate::celeste_map::{
    element::Element,
    lookup::Lookup,
};

pub mod attribute;
pub mod element;
pub mod header;
pub mod lookup;

#[derive(Debug, Clone)]
pub struct CelesteMap<'l, A: allocator_api2::alloc::Allocator = allocator_api2::alloc::Global> {
    pub tree: Element<'l, A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum CelesteMapReadError {
    Header(header::HeaderReadError),
    PackageName(dotnet_io_binary::io::string::ReadError),
    Lookup(lookup::LookupReadError),
    Element(element::ElementReadError),
}

impl<'l> CelesteMap<'l> {
    pub fn read<R: std::io::Read>(
        reader: R,
        lookup: &'l Lookup,
    ) -> Result<Self, CelesteMapReadError> {
        Self::read_in(Default::default(), reader, lookup)
    }
}

impl<'l, A: allocator_api2::alloc::Allocator + Clone> CelesteMap<'l, A> {
    pub fn read_package_name_in(
        alloc: A,
        mut reader: impl std::io::Read,
    ) -> Result<allocator_api2::boxed::Box<[u8], A>, CelesteMapReadError> {
        header::read_header(&mut reader)?;
        let package_name = crate::read_dotnet_str(alloc, &mut reader)?;
        Ok(package_name)
    }

    pub fn read_in<R: std::io::Read>(
        alloc: A,
        reader: R,
        lookup: &'l Lookup<A>,
    ) -> Result<Self, CelesteMapReadError> {
        let tree = Element::read(alloc, reader, lookup)?;
        Ok(Self { tree })
    }
}
