use crate::celeste_map::{
    attribute::Attribute,
    lookup::Lookup,
};

#[derive(Debug, Clone)]
pub struct Element<'l, A: allocator_api2::alloc::Allocator> {
    pub name: &'l [u8],
    pub attributes: allocator_api2::boxed::Box<[Attribute<'l, A>], A>,
    pub children: allocator_api2::boxed::Box<[Element<'l, A>], A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum ElementReadError {
    Io(std::io::Error),
    Attribute(super::attribute::AttributeReadError),
}

impl<'l, A: allocator_api2::alloc::Allocator + Clone> Element<'l, A> {
    pub(crate) fn read<R: std::io::Read>(
        alloc: A,
        mut reader: impl std::io::Read + core::borrow::BorrowMut<R>,
        lookup: &'l Lookup<A>,
    ) -> Result<Self, ElementReadError>
    where
        for<'a> &'a mut R: core::borrow::BorrowMut<R>,
    {
        use dotnet_io_binary::io::prim::ReadPrim;

        let name = lookup.read_indexed(reader.borrow_mut())?;
        let attr_count: u8 = reader.read_prim()?;
        let mut attributes =
            allocator_api2::boxed::Box::new_uninit_slice_in(attr_count as _, alloc.clone());
        for i in 0..attr_count {
            let attr = Attribute::read(alloc.clone(), reader.borrow_mut(), lookup)?;
            attributes[i as usize].write(attr);
        }
        let mut attributes = unsafe { attributes.assume_init() };
        attributes.sort();
        let child_count: u16 = reader.read_prim()?;
        let mut children =
            allocator_api2::boxed::Box::new_uninit_slice_in(child_count as _, alloc.clone());
        for i in 0..child_count {
            let elem = Element::read(alloc.clone(), reader.borrow_mut(), lookup)?;
            children[i as usize].write(elem);
        }
        let children = unsafe { children.assume_init() };
        Ok(Self {
            name,
            attributes,
            children,
        })
    }
}
