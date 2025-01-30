use std::any::TypeId;

use crate::force::ForceContainer;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct System {
    potential: ForceContainer,
    registered_backends: Vec<TypeId>,
    pub step: u64,
}

impl System {
    pub fn new() -> Self {
        Self::default()
    }
}
