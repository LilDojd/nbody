use std::any::{Any, TypeId};

use crate::{
    backend::Backend,
    helpers::{
        macros::{impl_box_clone, impl_ref_dyn_partialeq},
        DynCompare,
    },
    system::System,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ForceContainer {
    inner: Vec<Box<dyn ErasedForce>>,
}

impl ForceContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_force<B>(&mut self, force: impl ForceImpl<Backend = B>)
    where
        B: Backend + 'static,
    {
        self.inner.push(Box::new(force));
    }

    // A temporary function to show how we can leverage type system to assemble
    // forces from different backends.
    // One can imagine doing fold and map (reduce) here for a specific `B: Backend`
    pub fn compute_first_force<B: Backend>(&self, system: &System) -> B::Vector {
        let f = self.inner.first().unwrap();
        *f.compute_force(system).downcast::<B::Vector>().unwrap()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<F> ErasedForce for F
where
    F: ForceImpl + std::fmt::Debug + Clone + DynCompare + 'static,
{
    fn compute_force(&self, system: &System) -> Box<dyn Any> {
        Box::new(ForceImpl::force(self, system, ()))
    }

    fn compute_energy(&self, system: &System) -> Box<dyn Any> {
        Box::new(ForceImpl::energy(self, system, ()))
    }

    fn backend_id(&self) -> TypeId {
        TypeId::of::<F::Backend>()
    }
}

pub trait ErasedForce: std::fmt::Debug + DynCompare + BoxCloneErasedForce {
    /// Compute force for the backend this force is implemented for
    fn compute_force(&self, system: &System) -> Box<dyn Any>;
    /// Compute energy for the backend this force is implemented for
    fn compute_energy(&self, system: &System) -> Box<dyn Any>;
    /// Return the TypeId of the backend this force targets
    fn backend_id(&self) -> TypeId;
}

impl_box_clone!(ErasedForce, BoxCloneErasedForce, box_clone);
impl_ref_dyn_partialeq!(ErasedForce);

/// A trait defining the concrete implementation for the Force for a given `Backend`
pub trait ForceImpl: std::fmt::Debug + Clone + PartialEq + 'static {
    type Backend: Backend;
    fn force(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector;
    fn energy(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::CpuBackend;

    #[derive(Debug, Clone, PartialEq)]
    struct MyForce;

    impl ForceImpl for MyForce {
        type Backend = CpuBackend<f64>;
        fn force(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector {
            1.5
        }
        fn energy(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector {
            1.5
        }
    }

    impl ForceImpl for MySecondForce {
        type Backend = CpuBackend<i8>;
        fn force(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector {
            -1
        }
        fn energy(&self, system: &System, params: ()) -> <Self::Backend as Backend>::Vector {
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

        forces.add_force(my_cpu_force);
        println!(
            "After first: {}",
            forces.compute_first_force::<CpuBackend<f64>>(&system)
        );
        forces.add_force(my_second_force.clone());
        println!(
            "First after second: {}",
            forces.compute_first_force::<CpuBackend<f64>>(&system)
        );
        forces.clear();
        forces.add_force(my_second_force);
        println!(
            "Second after second: {}",
            forces.compute_first_force::<CpuBackend<i8>>(&system)
        );
        // Panics here
        println!(
            "{}",
            forces.compute_first_force::<CpuBackend<usize>>(&system)
        );
    }
}
