use sysinfo::System;
use std::cell::RefCell;
use std::rc::Rc;

use super::container_widget::{Container, Layout};
use super::progress_widget::ProgressBar;
use crate::Buffer;
use crate::widget::{Widget, Rect};

pub struct CpuInfo {
    sys: System,
    pub cpus_usage: Vec<Rc<RefCell<f64>>>,
    pub ram_used: Rc<RefCell<f64>>,
    pub ram_total: Rc<RefCell<f64>>,
    pub swap_used: Rc<RefCell<f64>>,
    pub swap_total: Rc<RefCell<f64>>,
    ui: Box<dyn Widget>
}

impl CpuInfo {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();

        let cpu_count = sys.cpus().len();

        let cpus_usage = (0..cpu_count)
            .map(|_| Rc::new(RefCell::new(0.0)))
            .collect::<Vec<_>>();
        
        let ram_used =  Rc::new(RefCell::new(0.0));
        let ram_total =  Rc::new(RefCell::new(1.0));
        let swap_used =  Rc::new(RefCell::new(0.0));
        let swap_total =  Rc::new(RefCell::new(1.0));
      
        // create ui 
        let mut ui = Container::new(None, None, Layout::Vertical, true);
        
        let total_progress = Rc::new(RefCell::new(100.0));
        for (i, cpu_usage) in cpus_usage.iter().enumerate() {
            let cpu_progress = ProgressBar::new(
                &format!("{:3}", i),
                "%",
                &cpu_usage,
                &total_progress);
            ui.add_child(Box::new(cpu_progress));
        }

        let ram_progress = ProgressBar::new("RAM", "MB", &ram_used, &ram_total);
        let swap_progress = ProgressBar::new("SWP", "MB", &swap_used, &swap_total);

        ui.add_child(Box::new(ram_progress));
        ui.add_child(Box::new(swap_progress));
        
        CpuInfo {
            sys,
            cpus_usage,
            ram_used, 
            ram_total, 
            swap_used, 
            swap_total, 
            ui: Box::new(ui) 
        }
        
    }

    fn update_data(&mut self) {
        self.sys.refresh_cpu_all();
        for (ind, cpu) in self.sys.cpus().iter().enumerate() {
            *self.cpus_usage[ind].borrow_mut() = cpu.cpu_usage() as f64;
        }

        self.sys.refresh_memory();
        *self.ram_used.borrow_mut() = (self.sys.used_memory() / 1024 / 1024) as f64;
        *self.ram_total.borrow_mut() = (self.sys.total_memory() / 1024 / 1024) as f64;

        *self.swap_used.borrow_mut() = (self.sys.used_swap() / 1024 / 1024) as f64;
        *self.swap_total.borrow_mut() = (self.sys.total_swap() / 1024 / 1024) as f64;
    }

}

impl Widget for CpuInfo {
    fn render(&self, buf: &mut Buffer, area: Rect) {
        self.ui.render(buf, area);
    }

    fn update(&mut self) {
        self.update_data();
        self.ui.update()
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(1), Some(1))
    }
}
