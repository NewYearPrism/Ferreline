#[derive(Debug, Clone, derive_more::Deref)]
#[deref(forward)]
pub struct Lookup<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator>(
    allocator_api2::vec::Vec<shared_vector::RefCountedVector<u8, Rc, A>, A>,
);

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum LookupReadError {
    Io(std::io::Error),
    ReadString(dotnet_io_binary::io::string::ReadError),
}

impl<A: allocator_api2::alloc::Allocator + Clone> Lookup<shared_vector::DefaultRefCount, A> {
    pub(crate) fn read<R: std::io::Read>(alloc: A, mut reader: R) -> Result<Self, LookupReadError> {
        use dotnet_io_binary::io::{
            prim::ReadPrim,
            string::ReadDotnetStr,
        };

        let count: i16 = reader.read_prim()?;
        let mut list = allocator_api2::vec::Vec::with_capacity_in(count as _, alloc.clone());
        for _ in 0..count {
            let buf = reader.read_dotnet_str(|len| {
                let mut buf = shared_vector::Vector::with_capacity_in(len as _, alloc.clone());
                buf.extend(core::iter::repeat_n(0, len as _));
                buf
            })?;
            list.push(buf.into());
        }
        Ok(Self(list))
    }
}

impl<A: allocator_api2::alloc::Allocator + Clone> Lookup<shared_vector::AtomicRefCount, A> {
    pub(crate) fn read_atomic<R: std::io::Read>(
        alloc: A,
        mut reader: R,
    ) -> Result<Self, LookupReadError> {
        use dotnet_io_binary::io::{
            prim::ReadPrim,
            string::ReadDotnetStr,
        };

        let count: i16 = reader.read_prim()?;
        let mut list = allocator_api2::vec::Vec::with_capacity_in(count as _, alloc.clone());
        for _ in 0..count {
            let buf = reader.read_dotnet_str(|len| {
                let mut buf = shared_vector::Vector::with_capacity_in(len as _, alloc.clone());
                buf.extend(core::iter::repeat_n(0, len as _));
                buf
            })?;
            list.push(buf.into());
        }
        Ok(Self(list))
    }
}

impl<Rc: shared_vector::RefCount, A: allocator_api2::alloc::Allocator> Lookup<Rc, A> {
    pub(crate) fn read_indexed(
        &self,
        mut reader: impl std::io::Read,
    ) -> std::io::Result<shared_vector::RefCountedVector<u8, Rc, A>> {
        use dotnet_io_binary::io::prim::ReadPrim;

        let i: i16 = reader.read_prim()?;
        let a = self.0[i as usize].new_ref();
        Ok(a)
    }
}
