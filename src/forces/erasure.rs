use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::{
    backend::Backend,
    helpers::{DynCompare, opaque::Opaque},
};

use super::ForceImpl;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ErasedForceWrapper<F, B, S>(pub F, pub PhantomData<(B, S)>);

impl<F, B, S> private::ErasedForce<S> for ErasedForceWrapper<F, B, S>
where
    F: ForceImpl<B, S> + std::fmt::Debug + Clone + DynCompare + 'static,
    B: Backend + 'static,
    S: std::fmt::Debug + Clone + PartialEq + 'static,
{
    fn compute_force(&self, storage: &S) -> Box<dyn Any> {
        Box::new(ForceImpl::force(&self.0, storage))
    }

    unsafe fn compute_force_into(&self, storage: &S, output: *mut Opaque) {
        unsafe {
            let vector = self.0.force(storage);
            let output = output as *mut B::Vector;
            *output = vector;
        }
    }

    fn backend_id(&self) -> TypeId {
        TypeId::of::<B>()
    }
}

pub(super) mod private {
    use std::any::{Any, TypeId};

    use crate::helpers::{
        DynCompare,
        macros::{impl_box_clone, impl_ref_dyn_partialeq},
    };

    use super::Opaque;

    pub(crate) trait ErasedForce<S>:
        std::fmt::Debug + DynCompare + BoxCloneErasedForce<S>
    {
        /// Compute force for the backend this force is implemented for
        fn compute_force(&self, storage: &S) -> Box<dyn Any>;
        /// Write result of calling compute_force into a pre-allocated memory
        ///
        /// Safety:
        /// 1. Pointer must be aligned and not-null
        /// 2. Caller must ensure that besides invariants required by MaybeUninit, pointer matches
        ///    the expected output B::Vector type
        /// 3. Memory at `output` must be initialized after returning
        /// 4. Caller must ensure that the data at `output` will get dropped
        unsafe fn compute_force_into(&self, storage: &S, output: *mut Opaque);
        /// Return the TypeId of the backend this force targets
        fn backend_id(&self) -> TypeId;
    }

    impl_box_clone!(ErasedForce<S>);
    impl_ref_dyn_partialeq!(ErasedForce<S>);
}
