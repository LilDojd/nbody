use device::DeviceOps;
mod cpu;
mod device;

pub use cpu::CpuBackend;

pub trait Backend: Clone + Default + Sized + Send + Sync + core::fmt::Debug + 'static {
    type Device: DeviceOps;

    type Vector;

    fn name() -> String;

    fn sync(_device: &Self::Device) {}
}
