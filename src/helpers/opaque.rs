/// A type-opaque struct to mimic C void*
#[repr(C)]
pub(crate) struct Opaque {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
