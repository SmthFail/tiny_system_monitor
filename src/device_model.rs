use once_cell::sync::Lazy;
use std::collections::HashMap;
use super::{cpu_device, gpu_device};
use super::app_config::DeviceTile;

pub trait Device {
    fn new(device_tile: &DeviceTile) -> Self where Self: Sized;
    fn resize(&mut self, width: u16, height: u16);
    fn update(&mut self);
    fn show(&mut self) -> &Vec<String>;
    fn get_name(&self) -> String;
}

type DeviceFactory = HashMap<&'static str, fn(&DeviceTile) -> Box<dyn Device>>; 

pub static DEVICE_REGISTRY: Lazy<DeviceFactory> = 
    Lazy::new(|| {
        let mut registry: DeviceFactory = HashMap::new();
        registry.insert(
            "cpu",
            cpu_device::create_device, 
        );
        registry.insert(
            "gpu",
            gpu_device::create_device
        );
    registry
});
