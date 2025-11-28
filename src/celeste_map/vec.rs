use allocator_api2::{
    alloc::Allocator,
    vec::Vec,
};

pub(crate) fn default_vec<A: Allocator + Default, T>() -> Vec<T, A> {
    Vec::new_in(Default::default())
}
