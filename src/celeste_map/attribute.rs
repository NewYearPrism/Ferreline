use crate::celeste_map::lookup::Lookup;

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum AttributeReadError {
    Io(std::io::Error),
    UnknownType(derive_more::TryFromReprError<u8>),
    ReadString(dotnet_io_binary::io::string::ReadError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::TryFrom)]
#[try_from(repr)]
#[repr(u8)]
enum AttributeType {
    Boolean = 0,
    Byte,
    Int16,
    Int32,
    Single,
    Lookup,
    Str,
    Rle,
}

impl AttributeType {
    fn read<R: std::io::Read + core::borrow::BorrowMut<R>>(
        mut reader: R,
    ) -> Result<Self, AttributeReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;
        let a: u8 = reader.read_prim()?;
        let t = a.try_into()?;
        Ok(t)
    }
}

#[derive(Debug, Clone)]
pub enum AttributeValue<'l, A: allocator_api2::alloc::Allocator> {
    Boolean(bool),
    Byte(u8),
    Int16(i16),
    Int32(i32),
    Single(f32),
    Lookup(&'l [u8]),
    Str(allocator_api2::boxed::Box<[u8], A>),
    Rle(allocator_api2::boxed::Box<[u8], A>),
}

impl AttributeType {
    fn read_value<
        A: allocator_api2::alloc::Allocator,
        R: std::io::Read + core::borrow::BorrowMut<R>,
    >(
        self,
        alloc: A,
        mut reader: R,
        lookup: &Lookup<A>,
    ) -> Result<AttributeValue<'_, A>, AttributeReadError> {
        use dotnet_io_binary::io::prim::ReadPrim;
        let attr = match self {
            AttributeType::Boolean => {
                let p: u8 = reader.read_prim()?;
                AttributeValue::Boolean(p != 0)
            }
            AttributeType::Byte => AttributeValue::Byte(reader.read_prim()?),
            AttributeType::Int16 => AttributeValue::Int16(reader.read_prim()?),
            AttributeType::Int32 => AttributeValue::Int32(reader.read_prim()?),
            AttributeType::Single => AttributeValue::Single(reader.read_prim()?),
            AttributeType::Lookup => AttributeValue::Lookup(lookup.read_indexed(reader)?),
            AttributeType::Str => {
                let buf = crate::read_dotnet_str(alloc, &mut reader)?;
                AttributeValue::Str(buf)
            }
            AttributeType::Rle => {
                let len: i16 = reader.read_prim()?;
                let mut buf = unsafe {
                    allocator_api2::boxed::Box::new_zeroed_slice_in(len as _, alloc).assume_init()
                };
                reader.read_exact(&mut buf)?;
                AttributeValue::Rle(buf)
            }
        };
        Ok(attr)
    }
}

impl<'l, A: allocator_api2::alloc::Allocator> AttributeValue<'l, A> {
    fn read<R: std::io::Read + core::borrow::BorrowMut<R>>(
        alloc: A,
        mut reader: R,
        lookup: &'l Lookup<A>,
    ) -> Result<Self, AttributeReadError> {
        let t = AttributeType::read(reader.borrow_mut())?;
        let a = t.read_value(alloc, reader, lookup)?;
        Ok(a)
    }
}

#[derive(Debug, Clone, derivative::Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct Attribute<'l, A: allocator_api2::alloc::Allocator> {
    pub name: &'l [u8],
    #[derivative(PartialEq = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub value: AttributeValue<'l, A>,
}

impl<'l, A: allocator_api2::alloc::Allocator> Attribute<'l, A> {
    pub(crate) fn read<R: std::io::Read + core::borrow::BorrowMut<R>>(
        alloc: A,
        mut reader: R,
        lookup: &'l Lookup<A>,
    ) -> Result<Self, AttributeReadError> {
        let name = lookup.read_indexed(reader.borrow_mut())?;
        let value = AttributeValue::read(alloc, reader, lookup)?;
        Ok(Self { name, value })
    }
}
