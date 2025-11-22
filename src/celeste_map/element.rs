use crate::celeste_map::{
    attribute::Attribute,
    lookup::Lookup,
};

#[derive(Debug, Clone)]
pub struct Element<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator> {
    pub name: shared_vector::RefCountedVector<u8, Rc, A>,
    pub attributes: allocator_api2::vec::Vec<Attribute<Rc, A>, A>,
    pub children: allocator_api2::vec::Vec<Element<Rc, A>, A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum ElementReadError {
    Io(std::io::Error),
    Attribute(super::attribute::AttributeReadError),
}

impl<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator + Clone> Element<Rc, A> {
    pub(crate) fn read<R: std::io::Read>(
        alloc: A,
        mut reader: impl std::io::Read + core::borrow::BorrowMut<R>,
        lookup: &Lookup<Rc, A>,
    ) -> Result<Self, ElementReadError>
    where
        for<'a> &'a mut R: core::borrow::BorrowMut<R>,
    {
        use dotnet_io_binary::io::prim::ReadPrim;

        let name = lookup.read_indexed(reader.borrow_mut())?;
        let attr_count: u8 = reader.read_prim()?;
        let mut attributes =
            allocator_api2::vec::Vec::with_capacity_in(attr_count as _, alloc.clone());
        for _ in 0..attr_count {
            let attr = Attribute::read(alloc.clone(), reader.borrow_mut(), lookup)?;
            attributes.push(attr);
        }
        attributes.sort_by(|a, b| a.name.cmp(&b.name));
        let child_count: u16 = reader.read_prim()?;
        let mut children =
            allocator_api2::vec::Vec::with_capacity_in(child_count as _, alloc.clone());
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

#[cfg(feature = "sync")]
unsafe impl<Rc: shared_vector::RefCount + Sync, A: allocator_api2::alloc::Allocator> Sync
    for Element<Rc, A>
{
}
