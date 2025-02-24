use crate::{
    backend::Backend,
    forces::{ForceContainer, ForceImpl},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct System<S: 'static> {
    potential: ForceContainer<S>,
    integrator: Integrator,
    pub step: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Integrator;

impl<S: Default> System<S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> System<S>
where
    S: Default + 'static + std::fmt::Debug + Clone + PartialEq,
{
    pub fn add_force<B>(&mut self, force: impl ForceImpl<B, S>)
    where
        B: Backend + 'static,
    {
        self.potential.add_force(force);
    }
}
