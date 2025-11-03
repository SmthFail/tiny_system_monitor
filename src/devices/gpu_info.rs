use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

use std::rc::Rc;
use std::cell::RefCell;

use crate::Buffer;
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use nvml_wrapper::{Nvml, error::NvmlError};
use nvml_wrapper::enum_wrappers::device::PcieUtilCounter;
use crate::tui::{
    container_widget::{Container, Layout, Alignment},
    progress_bar_widget::ProgressBar,
    text_widget::TextWidget,
    app_error::AppError,
    cell_types::CellString
};



use std::ffi::OsStr;

struct Gpu{
    index: u32,
    gpu_info: String,
    memory_used: Rc<RefCell<f64>>,
    memory_total: Rc<RefCell<f64>>,
    utilization_rate: Rc<RefCell<f64>>,
    temperature: f64,
    param_string: CellString
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
            Ok(temperature) => temperature as f64,
            Err(_err) => panic!("Can't read temperature"),
        };

        let memory_info = match device.memory_info() {
            Ok(memory_info) => memory_info,
            Err(_err) => panic!("{}", _err),
        };

        let tx = match device.pcie_throughput(PcieUtilCounter::Send) {
            Ok(tx) => tx,
            Err(_err) => panic!("{}", _err)
        };

        let rx = match device.pcie_throughput(PcieUtilCounter::Receive) {
            Ok(rx) => rx,
            Err(_err) => panic!("{}", _err)
        };

        let mut param_string = CellString::new();
        param_string.update(format!("T:{}℃ , Rx: {} KB/s, Tx: {}KB/s", temperature, rx, tx));

        let utilization_rate = match device.utilization_rates() {
            Ok(utilization_rates) => Rc::new(RefCell::new(utilization_rates.gpu as f64)),
            Err(_err) => panic!("{}", _err),
        };

        let memory_used = memory_info.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_total = memory_info.total as f64 / 1024.0 /1024.0 / 1024.0;

        Gpu {
            index, 
            gpu_info,
            temperature,
            memory_used: Rc::new(RefCell::new(memory_used)),
            memory_total: Rc::new(RefCell::new(memory_total)),
            utilization_rate,
            param_string
        }
    }

    fn update(&mut self, nvml: &Nvml) {
        let device = nvml.device_by_index(self.index).unwrap();
       
        *self.memory_used.borrow_mut() = match device.memory_info() {
            Ok(memory_info) => memory_info.used as f64 / 1024.0 / 1024.0 / 1024.0,
            Err(_err) => panic!("{}", _err),
        };

        *self.utilization_rate.borrow_mut() = match device.utilization_rates() {
            Ok(utilization_rates) => utilization_rates.gpu as f64,
            Err(_err) => panic!("{}", _err),
        };
       
        self.temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature as f64,
            Err(_err) => panic!("Can't read temperature"),
        };
 
        let rx = match device.pcie_throughput(PcieUtilCounter::Receive) {
            Ok(rx) => rx as f64,
            Err(_err) => panic!("{}", _err),
        };

        let tx = match device.pcie_throughput(PcieUtilCounter::Send) {
            Ok(tx) => tx as f64,
            Err(_err) => panic!("{}", _err),
        };
        
        self.param_string.update(format!("T:{}℃ , Rx: {} KB/s, Tx: {}KB/s", self.temperature, rx, tx));
    }
}



pub struct GpuInfo{
    nvml: Nvml,
    gpus: Vec<Gpu>,
    ui: Box<dyn Widget>
}

impl GpuInfo{
    pub fn new() -> Self {
        let nvml = Self::_search_nvml().unwrap_or_else(|err| {
            panic!("Error in load gpu driver: {}", err)
        });
        
        let device_count = nvml.device_count().unwrap();
        
        let mut gpus = Vec::new();
        for i in 0..device_count {
            let gpu = Gpu::new(i, &nvml);
            gpus.push(gpu)
        } 


        // create ui
        let mut ui = Container::new(None, None, Layout::Vertical, Alignment::Start);

        let mut gpus_container = Container::new(None, None, Layout::Grid, Alignment::Start);

        for gpu in gpus.iter() {
            // set height to 5(with 1 space) untill implement auto size of container
            let mut gpu_container = Container::new(None, Some(5), Layout::Vertical, Alignment::Start);
            
            let info_string = &format!("{}", &gpu.gpu_info);
            gpu_container.add_child(TextWidget::new_static(info_string));
            gpu_container.add_child(TextWidget::new_editable(gpu.param_string.clone()));
            gpu_container.add_child(ProgressBar::new(
                "GPU", "GB", &gpu.memory_used, &gpu.memory_total, false) 
            );
            
            let total_gpu = Rc::new(RefCell::new(100.0));
            gpu_container.add_child(ProgressBar::new(
                "RAM", "%", &gpu.utilization_rate, &total_gpu, true)
            );
            gpus_container.add_child(gpu_container);
        }

        ui.add_child(gpus_container);
        
        GpuInfo {
            nvml, 
            gpus,
            ui: Box::new(ui)
        }
    }

    fn _search_nvml() -> Result<Nvml, NvmlError> {
        if let Ok(nvml) = Nvml::init() {
            return Ok(nvml);
        }

        let candidates: &[&str] = &[
            "libnvidia-ml.so",            
            "libnvidia-ml.so.1",
        ]; 

        for &name in candidates {
            let lib_path = OsStr::new(name);
            match Nvml::builder().lib_path(lib_path).init() {
                Ok(nvml) => return Ok(nvml),
                Err(NvmlError::LibloadingError(_)) => continue,
                Err(e) => return Err(e)
            }
        }
        Err(NvmlError::LibraryNotFound)
    }

    pub fn update(&mut self) {
        for gpu in self.gpus.iter_mut() {
            gpu.update(&self.nvml)
        }
    }
}


impl Widget for GpuInfo {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        self.ui.render(buf, area)?;
        Ok(())
    }

    fn update(&mut self) -> Result<(), AppError>{
        self.update();
        self.ui.update()?;
        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
       ChildConstraints {
           min_width: None,
           max_width: None,
           min_height: None,
           max_height: None
       } 
    }
}
