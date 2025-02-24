/// A type-opaque struct to mimic C void*
#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Opaque {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
