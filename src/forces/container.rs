use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit};

use crate::{backend::Backend, helpers::opaque::Opaque, system::System};

use super::{
    erasure::{private::ErasedForce, ErasedForceWrapper},
    ForceImpl,
};

type BackendId = TypeId;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ForceContainer {
    inner: HashMap<BackendId, Vec<Box<dyn ErasedForce>>>,
}

impl ForceContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_force<B>(&mut self, force: impl ForceImpl<B>)
    where
        B: Backend + 'static,
    {
        let wrapped = Box::new(ErasedForceWrapper(force, PhantomData));
        let id = wrapped.backend_id();
        match self.inner.get_mut(&id) {
            Some(backvec) => backvec.push(wrapped),
            None => {
                self.inner.insert(id, vec![wrapped]);
            }
        }
    }

    // A temporary function to show how we can leverage type system to assemble
    // forces from different backends.
    // One can imagine doing fold and map (reduce) here for a specific `B: Backend`
    // Or processing each force and folding it to some unified representation
    pub fn compute_first_matching_force<B: Backend>(&self, system: &System) -> Option<B::Vector> {
        // Unwrap last since downcast is guaranteed to suceed
        self.inner.get(&TypeId::of::<B>()).map(|f| {
            f.first().map(|f| {
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
        })?
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
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
    }

    impl ForceImpl<CpuBackend<f32>> for MySecondForce {
        fn force(&self, system: &System, params: ()) -> f32 {
            -100.
        }
    }

    impl ForceImpl<CpuBackend<i8>> for MySecondForce {
        fn force(&self, system: &System, params: ()) -> i8 {
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
