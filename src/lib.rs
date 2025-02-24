pub mod backend;
pub mod forces;
mod helpers;
mod system;
mod vector;

/// Represents a pair of objects, which can be particles or storages of particles, between which an
/// interaction is computed.
///
/// The first object is the one being affected by the second object.
#[derive(Clone, Copy, Debug)]
pub struct Between<S1, S2>(pub S1, pub S2);
