use once_cell::sync::Lazy;
use std::collections::HashMap;
use super::{cpu_device, gpu_device};
use super::app_config::DeviceTile;

pub trait Device {
    fn new(device_tile: &DeviceTile) -> Self where Self: Sized;
    fn update(&mut self);
    fn show(&self);
}

pub struct DeviceFactory {
    pub name: &'static str,
    pub create: fn(device_tile: &DeviceTile) -> Box<dyn Device>,
}

pub static DEVICE_REGISTRY: Lazy<HashMap<&'static str, DeviceFactory>> = 
    Lazy::new(|| {
        let mut registry = HashMap::new();
        registry.insert(
            "cpu",
            DeviceFactory {
                name: "cpu",
                create: cpu_device::create_device,
            }
        );
        registry.insert(
            "gpu",
            DeviceFactory{
                name: "gpu",
                create: gpu_device::create_device,
            }
        );
    registry
});
