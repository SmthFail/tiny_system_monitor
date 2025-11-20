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
    fn new(index: u32, nvml: &Nvml) -> Result<Self, AppError> {
        let device = match nvml.device_by_index(index) {
            Ok(device) => device,
            Err(err) => return Err(AppError::error(format!("Cannot access GPU {}: {}", index, err))),
        };

        let name = match device.name() {
            Ok(name) => name,
            Err(err) => return Err(AppError::error(format!("Cannot read GPU device name: {:?}", err))),
        };

        let capability = match device.cuda_compute_capability() {
            Ok(compute_capability) => {
                format!("{}.{}", compute_capability.major, compute_capability.minor)
            }
            Err(_err) => "".to_string(),
        };

        let gpu_info = format!("{}, Cap: {}", name, capability);

        let temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature as f64,
            Err(err) => return Err(AppError::error(format!("Cannot read temperature: {}", err))),
        };

        let memory_info = match device.memory_info() {
            Ok(memory_info) => memory_info,
            Err(err) => return Err(AppError::error(format!("Cannot get memory info: {:?}", err))),
        };

        let tx = match device.pcie_throughput(PcieUtilCounter::Send) {
            Ok(tx) => tx,
            Err(err) => return Err(AppError::error(format!("Cannot get PCIE TX: {:?}", err)))
        };

        let rx = match device.pcie_throughput(PcieUtilCounter::Receive) {
            Ok(rx) => rx,
            Err(err) => return Err(AppError::error(format!("Cannot get PCIE RX: {:?}", err)))
        };

        let mut param_string = CellString::new();
        param_string.update(format!("T:{}℃ , Rx: {} KB/s, Tx: {}KB/s", temperature, rx, tx));

        let utilization_rate = match device.utilization_rates() {
            Ok(utilization_rates) => Rc::new(RefCell::new(utilization_rates.gpu as f64)),
            Err(err) => return Err(AppError::error(format!("Cannot get utilization rates: {:?}", err))),
        };

        let memory_used = memory_info.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_total = memory_info.total as f64 / 1024.0 /1024.0 / 1024.0;

        Ok(Gpu {
            index,
            gpu_info,
            temperature,
            memory_used: Rc::new(RefCell::new(memory_used)),
            memory_total: Rc::new(RefCell::new(memory_total)),
            utilization_rate,
            param_string
        })
    }

    fn update(&mut self, nvml: &Nvml) -> Result<(), AppError> {
        let device = match nvml.device_by_index(self.index) {
            Ok(device) => device,
            Err(err) => return Err(AppError::error(format!("Cannot access GPU {}: {}", self.index, err))),
        };

        *self.memory_used.borrow_mut() = match device.memory_info() {
            Ok(memory_info) => memory_info.used as f64 / 1024.0 / 1024.0 / 1024.0,
            Err(err) => return Err(AppError::error(format!("Cannot get memory info: {:?}", err))),
        };

        *self.utilization_rate.borrow_mut() = match device.utilization_rates() {
            Ok(utilization_rates) => utilization_rates.gpu as f64,
            Err(err) => return Err(AppError::error(format!("Cannot get utilization rates: {:?}", err))),
        };

        self.temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature as f64,
            Err(err) => return Err(AppError::error(format!("Cannot read temperature: {}", err))),
        };

        let rx = match device.pcie_throughput(PcieUtilCounter::Receive) {
            Ok(rx) => rx as f64,
            Err(err) => return Err(AppError::error(format!("Cannot get PCIE RX: {:?}", err))),
        };

        let tx = match device.pcie_throughput(PcieUtilCounter::Send) {
            Ok(tx) => tx as f64,
            Err(err) => return Err(AppError::error(format!("Cannot get PCIE TX: {:?}", err))),
        };

        self.param_string.update(format!("T:{}℃ , Rx: {} KB/s, Tx: {}KB/s", self.temperature, rx, tx));
        Ok(())
    }
}



pub struct GpuInfo{
    nvml: Option<Nvml>,
    gpus: Vec<Gpu>,
    ui: Box<dyn Widget>,
    initialization_error: Option<AppError>,
}

impl GpuInfo{
    pub fn new() -> Self {
        // Try to initialize but gracefully handle errors
        match Self::_try_new() {
            Ok(gpu_info) => gpu_info,
            Err(error) => {
                // Create a fallback state when initialization fails
                GpuInfo {
                    nvml: None,
                    gpus: Vec::new(),
                    ui: Box::new(Container::new(None, None, Layout::Vertical, Alignment::Start)),
                    initialization_error: Some(error),
                }
            }
        }
    }

    fn _try_new() -> Result<Self, AppError> {
        let nvml = match Self::_search_nvml() {
            Ok(nvml) => nvml,
            Err(err) => return Err(AppError::error(format!("Error in load gpu driver: {}", err))),
        };

        let device_count = match nvml.device_count() {
            Ok(count) => count,
            Err(err) => return Err(AppError::error(format!("Error getting device count: {}", err))),
        };

        let mut gpus = Vec::new();
        for i in 0..device_count {
            let gpu = match Gpu::new(i, &nvml) {
                Ok(g) => g,
                Err(err) => return Err(AppError::error(format!("Error creating GPU {}: {}", i, err.message))),
            };
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

        Ok(GpuInfo {
            nvml: Some(nvml),
            gpus,
            ui: Box::new(ui),
            initialization_error: None,
        })
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

    pub fn update(&mut self) -> Result<(), AppError> {
        if let Some(ref error) = self.initialization_error {
            return Err(error.clone());
        }

        if let Some(ref nvml) = self.nvml {
            for gpu in self.gpus.iter_mut() {
                gpu.update(nvml)?;
            }
            // Update the UI
            self.ui.update()?;
        } else {
            // If there's no NVML (due to initialization error), just update the UI container
            self.ui.update()?;
        }

        Ok(())
    }
}


impl Widget for GpuInfo {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        if let Some(ref error) = self.initialization_error {
            // If there was an initialization error, show an error widget instead
            use crate::tui::widgets::error_widget::ErrorWidget;
            let mut error_widget = ErrorWidget::new(error);
            return error_widget.render(buf, area);
        }

        self.ui.render(buf, area)
    }

    fn update(&mut self) -> Result<(), AppError>{
        // Call the struct's update method which handles the error state
        self.update()
    }

    fn get_constraints(&self) -> ChildConstraints {
        // Always return flexible constraints, the error widget will handle sizing internally
        ChildConstraints {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None
        }
    }
}
