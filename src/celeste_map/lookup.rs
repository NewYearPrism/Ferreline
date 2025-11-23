#[derive(Debug, Clone, derive_more::Deref)]
#[deref(forward)]
pub struct Lookup<A: allocator_api2::alloc::Allocator = allocator_api2::alloc::Global>(
    allocator_api2::boxed::Box<[allocator_api2::boxed::Box<[u8], A>], A>,
);

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum LookupReadError {
    Io(std::io::Error),
    ReadString(dotnet_io_binary::io::string::ReadError),
}

impl<A: allocator_api2::alloc::Allocator + Clone> Lookup<A> {
    pub fn read_in<R: std::io::Read>(alloc: A, mut reader: R) -> Result<Self, LookupReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;

        let count: i16 = reader.read_prim()?;
        let mut list = allocator_api2::boxed::Box::new_uninit_slice_in(count as _, alloc.clone());
        for i in 0..count {
            let buf = crate::read_dotnet_str(alloc.clone(), &mut reader)?;
            list[i as usize].write(buf);
        }
        let list = unsafe { list.assume_init() };
        Ok(Self(list))
    }
}

impl<A: allocator_api2::alloc::Allocator> Lookup<A> {
    pub(crate) fn read_indexed(&self, mut reader: impl std::io::Read) -> std::io::Result<&[u8]> {
        use dotnet_io_binary::io::prim::ReadPrim;

        let i: i16 = reader.read_prim()?;
        let a = &self.0[i as usize];
        Ok(a)
    }
}
