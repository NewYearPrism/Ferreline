use crate::celeste_map::{
    element::Element,
    lookup::Lookup,
};

pub mod attribute;
pub mod element;
pub mod header;
pub mod lookup;

#[derive(Debug, Clone)]
pub struct CelesteMap<
    Rc: shared_vector::RefCount = shared_vector::DefaultRefCount,
    A: allocator_api2::alloc::Allocator = allocator_api2::alloc::Global,
> {
    pub package_name: allocator_api2::vec::Vec<u8, A>,
    pub lookup: Lookup<Rc, A>,
    pub tree: Element<Rc, A>,
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum CelesteMapReadError {
    Header(header::HeaderReadError),
    PackageName(dotnet_io_binary::io::string::ReadError),
    Lookup(lookup::LookupReadError),
    Element(element::ElementReadError),
}

impl CelesteMap {
    pub fn read<R: std::io::Read + core::borrow::BorrowMut<R>>(
        reader: R,
    ) -> Result<Self, CelesteMapReadError> {
        Self::read_in(Default::default(), reader)
    }
}

impl<A: allocator_api2::alloc::Allocator + Clone> CelesteMap<shared_vector::DefaultRefCount, A> {
    pub fn read_in<R: std::io::Read + core::borrow::BorrowMut<R>>(
        alloc: A,
        mut reader: R,
    ) -> Result<Self, CelesteMapReadError> {
        Self::with_rc_in(alloc, &mut reader)
    }
}

impl<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator + Clone> CelesteMap<Rc, A> {
    pub fn with_rc_in<R: std::io::Read + core::borrow::BorrowMut<R>>(
        alloc: A,
        mut reader: R,
    ) -> Result<Self, CelesteMapReadError> {
        use dotnet_io_binary::io::string::ReadDotnetStr;

        header::read_header(reader.borrow_mut())?;
        let package_name = reader.read_dotnet_str(|len| {
            let mut buf = allocator_api2::vec::Vec::with_capacity_in(len as _, alloc.clone());
            buf.extend(core::iter::repeat_n(0, len as _));
            buf
        })?;
        let lookup = Lookup::read(alloc.clone(), reader.borrow_mut())?;
        let tree = Element::read(alloc.clone(), reader, &lookup)?;
        Ok(Self {
            package_name,
            lookup,
            tree,
        })
    }
}

#[cfg(feature = "visit")]
pub mod visit {
    use directed_visit::Director;

    use crate::celeste_map::element::Element;

    pub struct ElementDirector;

    impl<
        Rc: shared_vector::RefCount,
        A: allocator_api2::alloc::Allocator,
        V: directed_visit::Visit<Element<Rc, A>>,
    > directed_visit::Direct<V, Element<Rc, A>> for ElementDirector
    {
        fn direct(mut director: Director<'_, Self, V>, node: &Element<Rc, A>) {
            for child in &node.children {
                Director::direct(&mut director, &child)
            }
        }
    }
}
