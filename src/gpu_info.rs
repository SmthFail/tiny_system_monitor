use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

use std::rc::Rc;
use std::cell::RefCell;

use crate::Buffer;
use crate::widget::{Widget, Rect};
use nvml_wrapper::Nvml;
use crate::container_widget::{Container, Layout};
use super::progress_widget::ProgressBar;
use super::text_widget::TextWidget;

struct Gpu{
    index: u32,
    gpu_info: String,
    memory_used: Rc<RefCell<f64>>,
    memory_total: Rc<RefCell<f64>>,
    utilization_rate: Rc<RefCell<f64>>,
    temperature: u32,
}

impl Gpu {
    fn new(index: u32, nvml: &Nvml) -> Self {
        let device = nvml.device_by_index(index).unwrap();
        let name: String = device.name().expect("Can't read GPU device name");
        let capability = match device.cuda_compute_capability() {
            Ok(compute_capability) => {
                format!("{}.{}", compute_capability.major, compute_capability.minor)
            }
            Err(_err) => "".to_string(),
        };

        let gpu_info = format!("{}, Cap: {}", name, capability);
        
        let temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature,
            Err(_err) => panic!("Can't read temperature"),
        };

        let memory_info = match device.memory_info() {
            Ok(memory_info) => memory_info,
            Err(_err) => panic!("{}", _err),
        };

        let utilization_rate = match device.utilization_rates() {
            Ok(utilization_rates) => Rc::new(RefCell::new(utilization_rates.gpu as f64)),
            Err(_err) => panic!("{}", _err),
        };

        Gpu {
            index, 
            gpu_info,
            temperature,
            memory_used: Rc::new(RefCell::new(memory_info.used as f64)),
            memory_total: Rc::new(RefCell::new(memory_info.total as f64)),
            utilization_rate
        }
    }

    fn update(&mut self, nvml: &Nvml) {
        let device = nvml.device_by_index(self.index).unwrap();
        self.temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature,
            Err(_err) => panic!("Can't read temperature"),
        };

        *self.memory_used.borrow_mut() = match device.memory_info() {
            Ok(memory_info) => memory_info.used as f64,
            Err(_err) => panic!("{}", _err),
        };

        *self.utilization_rate.borrow_mut() = match device.utilization_rates() {
            Ok(utilization_rates) => utilization_rates.gpu as f64,
            Err(_err) => panic!("{}", _err),
        };

    }
}



pub struct GpuInfo{
    nvml: Nvml,
    gpus: Vec<Gpu>,
    ui: Box<dyn Widget>
}

impl GpuInfo{
    pub fn new() -> Self {
        let nvml = Nvml::init().unwrap(); // TODO handle error
        let device_count = nvml.device_count().unwrap();
        
        
        let mut gpus = Vec::new();
        for i in 0..device_count {
            let gpu = Gpu::new(i, &nvml);
            gpus.push(gpu)
        } 


        // create ui
        let mut ui = Container::new(None, None, Layout::Vertical, true);

        for gpu in gpus.iter() {

            let mut gpu_container = Container::new(None, None, Layout::Vertical, false);

            gpu_container.add_child(Box::new(TextWidget::new(&gpu.gpu_info)));
            gpu_container.add_child(Box::new(ProgressBar::new(
                "GPU", "Mb", &gpu.memory_used, &gpu.memory_total) 
            ));
            
            let total_gpu = Rc::new(RefCell::new(100.0));
            gpu_container.add_child(Box::new(ProgressBar::new(
                "RAM", "Mb", &gpu.utilization_rate, &total_gpu)
            ));
            ui.add_child(Box::new(gpu_container));
        }
        
        GpuInfo {
            nvml, 
            gpus,
            ui: Box::new(ui)
        }
    }

    pub fn update(&mut self) {
        for gpu in self.gpus.iter_mut() {
            gpu.update(&self.nvml)
        }
    }
}


impl Widget for GpuInfo {
    fn render(&self, buf: &mut Buffer, area: Rect) {
        self.ui.render(buf, area);
    }

    fn update(&mut self) {
        self.update();
        self.ui.update()
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(1), Some(1))
    }
}
