use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use crate::{backend::Backend, helpers::opaque::Opaque, system::System};

use super::{
    ForceImpl,
    erasure::{ErasedForceWrapper, private::ErasedForce},
};

type BackendId = TypeId;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ForceContainer<S>
where
    dyn ErasedForce<S>: PartialEq,
{
    inner: HashMap<BackendId, Vec<Box<dyn ErasedForce<S>>>>,
}

impl<S> ForceContainer<S>
where
    S: Default + 'static + std::fmt::Debug + Clone + PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }

    fn register_backend<B>(&mut self) {}

    pub fn add_force<B>(&mut self, force: impl ForceImpl<B, S>)
    where
        B: Backend + 'static,
    {
        let wrapped = Box::new(ErasedForceWrapper(force, PhantomData));
        let id = wrapped.backend_id();
        match self.inner.get_mut(&id) {
            Some(backvec) => backvec.push(wrapped),
            None => {
                self.register_backend::<B>();
                self.inner.insert(id, vec![wrapped]);
            }
        }
    }

    // A temporary function to show how we can leverage type system to assemble
    // forces from different backends.
    // One can imagine doing fold and map (reduce) here for a specific `B: Backend`
    // Or processing each force and folding it to some unified representation
    fn compute_matching_forces<B: Backend>(&self, storage: &S) -> Option<B::Vector>
    where
        B::Vector: Default + std::ops::AddAssign,
    {
        self.inner.get(&TypeId::of::<B>()).map(|forces| {
            forces.iter().fold(B::Vector::default(), |mut acc, force| {
                let mut result = std::mem::MaybeUninit::<B::Vector>::uninit();
                // SAFETY:
                // - The output pointer is valid and aligned.
                // - `compute_force_into` writes a valid B::Vector into the pointer.
                unsafe {
                    force.compute_force_into(storage, result.as_mut_ptr() as *mut Opaque);
                    acc += result.assume_init();
                }
                acc
            })
        })
    }

    pub fn compute_forces<V>(&self, storage: &S) -> V {
        self.inner.iter().map(|(tid, vf)| {
            vf.iter().map(|f| {
                let force = f.compute_force(storage);
                todo!()
            })
        });
        todo!()
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

    impl<S> ForceImpl<CpuBackend<f64>, S> for MyForce {
        fn force(&self, storage: &S) -> f64 {
            1.5
        }
    }

    impl<S> ForceImpl<CpuBackend<f32>, S> for MySecondForce {
        fn force(&self, storage: &S) -> f32 {
            -100.
        }
    }

    impl<S> ForceImpl<CpuBackend<i8>, S> for MySecondForce {
        fn force(&self, storage: &S) -> i8 {
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
        let system: System<()> = System::new();

        // We can add first force like this, since only one backend impl is present
        forces.add_force(my_cpu_force);
        // We then can query this impl and know the returning type
        assert_eq!(
            forces.compute_matching_forces::<CpuBackend<f64>>(&system),
            Some(1.5f64)
        );
        // Unsuccessful queries will return None
        assert_eq!(
            forces.compute_matching_forces::<CpuBackend<i64>>(&system),
            None
        );

        // Now let's add second force and query for its backend
        // Note the turbofish here, since we have multiple backends defined for this force
        // User must chose (or we will later fallback to most optimal backend with some proxy
        // Dynamic backend)
        forces.add_force::<CpuBackend<i8>>(my_second_force.clone());

        assert_eq!(
            forces.compute_matching_forces::<CpuBackend<i8>>(&system),
            Some(-1)
        );

        // We can add same force with the different backend also
        assert_eq!(
            forces.compute_matching_forces::<CpuBackend<f32>>(&system),
            None
        );
        forces.add_force::<CpuBackend<f32>>(my_second_force.clone());
        assert_eq!(
            forces.compute_matching_forces::<CpuBackend<f32>>(&system),
            Some(-100.)
        );

        // It just works!
    }
}
