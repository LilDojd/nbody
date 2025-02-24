use std::iter::Sum;

use device::DeviceOps;
mod cpu;
mod device;

pub use cpu::CpuBackend;

pub trait Backend:
    Clone + Default + Sized + Send + Sync + core::fmt::Debug + PartialEq + 'static
{
    type Device: DeviceOps;

    type Vector: Sum;

    fn name() -> String;

    fn sync(_device: &Self::Device) {}
}
