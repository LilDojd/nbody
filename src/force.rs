use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    mem::MaybeUninit,
};

use private::{ErasedForce, Opaque};

use crate::{backend::Backend, helpers::DynCompare, system::System};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ForceContainer {
    inner: Vec<Box<dyn ErasedForce>>,
}

/// A trait defining the concrete implementation for the Force for a given `Backend`
pub trait ForceImpl<B: Backend>: std::fmt::Debug + Clone + PartialEq + 'static {
    fn force(&self, system: &System, params: ()) -> B::Vector;
    fn energy(&self, system: &System, params: ()) -> B::Vector;
}

impl ForceContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_force<B>(&mut self, force: impl ForceImpl<B>)
    where
        B: Backend + 'static,
    {
        let wrapped = ErasedForceWrapper(force, PhantomData);
        self.inner.push(Box::new(wrapped));
    }

    // A temporary function to show how we can leverage type system to assemble
    // forces from different backends.
    // One can imagine doing fold and map (reduce) here for a specific `B: Backend`
    // Or processing each force and folding it to some unified representation
    pub fn compute_first_matching_force<B: Backend>(&self, system: &System) -> Option<B::Vector> {
        // Unwrap last since downcast is guaranteed to suceed
        self.inner
            .iter()
            .find(|f| f.backend_id() == TypeId::of::<B>())
            .map(|f| {
                let mut result = MaybeUninit::<B::Vector>::uninit();
                // SAFETY:
                // The output pointer is valid and matches the expected B::Vector type.
                // Pointer is aligned
                //
                // We use a c_style opaque type (wrapper around an empty array) to model void*
                // c_void would also probably do, but I am not sure
                unsafe {
                    f.compute_force_into(system, result.as_mut_ptr() as *mut Opaque);
                    result.assume_init() // ensure no leak
                }
            })
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ErasedForceWrapper<F, B>(F, PhantomData<B>);

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

    fn compute_energy(&self, system: &System) -> Box<dyn Any> {
        Box::new(ForceImpl::energy(&self.0, system, ()))
    }

    unsafe fn compute_energy_into(&self, system: &System, output: *mut Opaque) {
        let vector = self.0.energy(system, ());
        let output = output as *mut B::Vector;
        *output = vector;
    }

    fn backend_id(&self) -> TypeId {
        TypeId::of::<B>()
    }
}

mod private {
    use std::any::{Any, TypeId};

    use crate::{
        helpers::{
            macros::{impl_box_clone, impl_ref_dyn_partialeq},
            DynCompare,
        },
        system::System,
    };

    #[repr(C)]
    pub struct Opaque {
        _data: [u8; 0],
        _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
    }

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
        /// Compute energy for the backend this force is implemented for
        fn compute_energy(&self, system: &System) -> Box<dyn Any>;
        /// Write result of calling compute_energy into a pre-allocated memory
        ///
        /// Safety:
        /// 1. Pointer must be aligned and not-null
        /// 2. Caller must ensure that besides invariants required by MaybeUninit, pointer matches
        ///    the expected output B::Vector type
        /// 3. Memory at `output` must be initialized after returning
        /// 4. Caller must ensure that the data at `output` will get dropped
        unsafe fn compute_energy_into(&self, system: &System, output: *mut Opaque);
        /// Return the TypeId of the backend this force targets
        fn backend_id(&self) -> TypeId;
    }

    impl_box_clone!(ErasedForce, BoxCloneErasedForce, box_clone);
    impl_ref_dyn_partialeq!(ErasedForce);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::CpuBackend;

    #[derive(Debug, Clone, PartialEq)]
    struct MyForce;

    impl ForceImpl<CpuBackend<f64>> for MyForce {
        fn force(&self, system: &System, params: ()) -> f64 {
            1.5
        }
        fn energy(&self, system: &System, params: ()) -> f64 {
            1.5
        }
    }

    impl ForceImpl<CpuBackend<f32>> for MySecondForce {
        fn force(&self, system: &System, params: ()) -> f32 {
            -100.
        }
        fn energy(&self, system: &System, params: ()) -> f32 {
            -100.
        }
    }

    impl ForceImpl<CpuBackend<i8>> for MySecondForce {
        fn force(&self, system: &System, params: ()) -> i8 {
            -1
        }
        fn energy(&self, system: &System, params: ()) -> i8 {
            -1
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MySecondForce;

    #[test]
    fn test_add_force() {
        let my_cpu_force = MyForce;
        let my_second_force = MySecondForce;

        let mut forces = ForceContainer::new();
        let system = System::new();

        // We can add first force like this, since only one backend impl is present
        forces.add_force(my_cpu_force);
        // We then can query this impl and know the returning type
        assert_eq!(
            forces.compute_first_matching_force::<CpuBackend<f64>>(&system),
            Some(1.5f64)
        );
        // Unsuccessful queries will return None
        assert_eq!(
            forces.compute_first_matching_force::<CpuBackend<String>>(&system),
            None
        );

        // Now let's add second force and query for its backend
        // Note the turbofish here, since we have multiple backends defined for this force
        // User must chose (or we will later fallback to most optimal backend with some proxy
        // Dynamic backend)
        forces.add_force::<CpuBackend<i8>>(my_second_force.clone());

        assert_eq!(
            forces.compute_first_matching_force::<CpuBackend<i8>>(&system),
            Some(-1)
        );

        // We can add same force with the different backend also
        assert_eq!(
            forces.compute_first_matching_force::<CpuBackend<f32>>(&system),
            None
        );
        forces.add_force::<CpuBackend<f32>>(my_second_force.clone());
        assert_eq!(
            forces.compute_first_matching_force::<CpuBackend<f32>>(&system),
            Some(-100.)
        );

        // It just works!
    }
}
