use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::{
    backend::Backend,
    helpers::{opaque::Opaque, DynCompare},
    system::System,
};

use super::ForceImpl;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ErasedForceWrapper<F, B>(pub F, pub PhantomData<B>);

impl<F, B> private::ErasedForce for ErasedForceWrapper<F, B>
where
    F: ForceImpl<B> + std::fmt::Debug + Clone + DynCompare + 'static,
    B: Backend + 'static,
{
    fn compute_force(&self, system: &System) -> Box<dyn Any> {
        Box::new(ForceImpl::force(&self.0, system, ()))
    }

    unsafe fn compute_force_into(&self, system: &System, output: *mut Opaque) {
        let vector = self.0.force(system, ());
        let output = output as *mut B::Vector;
        *output = vector;
    }

    fn backend_id(&self) -> TypeId {
        TypeId::of::<B>()
    }
}

pub(super) mod private {
    use std::any::{Any, TypeId};

    use crate::{
        helpers::{
            macros::{impl_box_clone, impl_ref_dyn_partialeq},
            DynCompare,
        },
        system::System,
    };

    use super::Opaque;

    pub(crate) trait ErasedForce:
        std::fmt::Debug + DynCompare + BoxCloneErasedForce
    {
        /// Compute force for the backend this force is implemented for
        fn compute_force(&self, system: &System) -> Box<dyn Any>;
        /// Write result of calling compute_force into a pre-allocated memory
        ///
        /// Safety:
        /// 1. Pointer must be aligned and not-null
        /// 2. Caller must ensure that besides invariants required by MaybeUninit, pointer matches
        ///    the expected output B::Vector type
        /// 3. Memory at `output` must be initialized after returning
        /// 4. Caller must ensure that the data at `output` will get dropped
        unsafe fn compute_force_into(&self, system: &System, output: *mut Opaque);
        /// Return the TypeId of the backend this force targets
        fn backend_id(&self) -> TypeId;
    }

    impl_box_clone!(ErasedForce, BoxCloneErasedForce, box_clone);
    impl_ref_dyn_partialeq!(ErasedForce);
}
