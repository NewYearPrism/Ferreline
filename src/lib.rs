pub mod celeste_map;

pub(crate) fn read_dotnet_str<A: allocator_api2::alloc::Allocator>(
    alloc: A,
    mut reader: impl std::io::Read,
) -> Result<allocator_api2::boxed::Box<[u8], A>, dotnet_io_binary::io::string::ReadError> {
    use dotnet_io_binary::io::string::ReadDotnetStr;

    let buf = reader.read_dotnet_str(|len| {
        let buf = allocator_api2::boxed::Box::new_zeroed_slice_in(len as _, alloc);
        unsafe { buf.assume_init() }
    })?;
    Ok(buf)
}
