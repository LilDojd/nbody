#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DeviceId {
    pub type_id: u16,
    pub index_id: u32,
}

impl DeviceId {
    pub fn new(type_id: u16, index_id: u32) -> Self {
        DeviceId { type_id, index_id }
    }
}

pub trait DeviceOps: Clone + Default + PartialEq + Send + Sync + core::fmt::Debug {
    fn id(&self) -> DeviceId;
}

impl core::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
