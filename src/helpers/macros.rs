/// A simple macro to implement Clone for Box<dyn Trait>, without requiring that
/// Trait is Clone. This works by creating a new trait (`BoxCloneTrait`) and
/// making the first Trait inherit the `BoxCloneTrait`. `BoxCloneTrait` is
/// automatically implemented for all `T: Clone + Trait`.
///
/// Usage:
///
/// ```ignore
/// trait Foo: BoxCloneFoo {}
///
/// impl_box_clone!(Foo, BoxCloneFoo, box_clone_foo);
/// ```
macro_rules! impl_box_clone {
    ($Trait:ident $(<$($gen:tt),+>)?) => {
        paste::paste! {
        #[doc(hidden)]
        /// This is an internal implementation detail for cloning `Box<Trait>`
        pub trait [<BoxClone$Trait>] $(<$($gen),+>)? {
            /// Get a clone of self as a boxed trait.
            fn box_clone(&self) -> Box<dyn $Trait $(<$($gen),+>)? >;
        }}

        paste::paste! {

        #[doc(hidden)]
        /// This is an internal implementation detail for cloning `Box<Trait>`
        impl<$($($gen), +,)? T: Clone + $Trait $(<$($gen),+>)? + 'static> [<BoxClone$Trait>] $(<$($gen),+>)? for T {
            fn box_clone(&self) -> Box<dyn $Trait $(<$($gen),+>)? > {
                Box::new(self.clone())
            }
        }}

        impl $(<$($gen),+>)? Clone for Box<dyn $Trait $(<$($gen),+>)? > {
            fn clone(&self) -> Box<dyn $Trait $(<$($gen),+>)? > {
                self.box_clone()
            }
        }
    };
}
pub(crate) use impl_box_clone;

/// A simple macro to implement PartialEq for &dyn Trait, without requiring that
/// Trait is PartialEq. This works by having a `DynCompare` and implementing
/// `PartialEq` for it in a universal manner.
///
/// We can then have a blanket supertrait implementation via `AsDynCompare` for
/// base traits that satisfy `static bound
///
/// Usage:
///
/// ```ignore
/// trait Bar {};
/// impl_ref_dyn_partialeq!(Bar);
/// ```
macro_rules! impl_ref_dyn_partialeq {
    ($Trait: ident $(<$($gen:tt),+>)?) => {
        impl $(<$($gen),+>)?  PartialEq<dyn $Trait $(<$($gen),+>)?> for dyn $Trait $(<$($gen),+>)?
        $(where $($gen),+: 'static)? {
            fn eq(&self, other: &dyn $Trait $(<$($gen),+>)?) -> bool {
                self.as_dyn_compare() == other.as_dyn_compare()
            }
        }
    };
}
pub(crate) use impl_ref_dyn_partialeq;
