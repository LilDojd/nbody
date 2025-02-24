use std::{iter::Sum, marker::PhantomData};

use super::{
    Backend,
    device::{DeviceId, DeviceOps},
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct CpuBackend<V> {
    _v: PhantomData<V>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum CPUDevice {
    #[default]
    Cpu,
}

impl DeviceOps for CPUDevice {
    fn id(&self) -> DeviceId {
        match self {
            CPUDevice::Cpu => DeviceId::new(0, 0),
        }
    }
}

impl<V> Backend for CpuBackend<V>
where
    V: Sum + Default + Sync + Send + std::fmt::Debug + Clone + PartialEq + 'static,
{
    type Device = CPUDevice;
    type Vector = V;

    fn name() -> String {
        String::from("CPU")
    }
}
