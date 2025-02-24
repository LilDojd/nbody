use crate::backend::Backend;

mod container;
pub(crate) use container::ForceContainer;
pub(super) mod erasure;

/// A trait defining the concrete implementation for the Force for a given `Backend`
pub trait ForceImpl<B: Backend, S>: std::fmt::Debug + Clone + PartialEq + 'static {
    fn force(&self, storage: &S) -> B::Vector;
}
