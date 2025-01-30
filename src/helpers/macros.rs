/// A simple macro to implement Clone for Box<Trait>, without requiring that
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
    ($Trait: ident, $BoxClone: ident, $box_clone: ident) => {
        #[doc(hidden)]
        /// This is an internal implementation detail for cloning `Box<Trait>`
        pub trait $BoxClone {
            /// Get a cloned of self as a boxed trait.
            fn $box_clone(&self) -> Box<dyn $Trait>;
        }

        impl<T: Clone + $Trait + 'static> $BoxClone for T {
            fn $box_clone(&self) -> Box<dyn $Trait> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn $Trait> {
            fn clone(&self) -> Box<dyn $Trait> {
                self.$box_clone()
            }
        }
    };
}
pub(crate) use impl_box_clone;

macro_rules! impl_ref_dyn_partialeq {
    ($Trait: ident) => {
        impl PartialEq<dyn $Trait> for dyn $Trait {
            fn eq(&self, other: &dyn $Trait) -> bool {
                self.as_dyn_compare() == other.as_dyn_compare()
            }
        }
    };
}
pub(crate) use impl_ref_dyn_partialeq;
