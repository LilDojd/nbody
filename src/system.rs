use std::any::TypeId;

use crate::forces::ForceContainer;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct System {
    potential: ForceContainer,
    pub step: u64,
}

impl System {
    pub fn new() -> Self {
        Self::default()
    }
}
