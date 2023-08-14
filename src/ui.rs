use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::{execute, terminal};

use std::io::{stdout, Stdout};

use crate::gpu_info;
use gpu_info::GpuAll;

use crate::cpu_info;
use cpu_info::CpuInfo;

pub fn calculate_progress_bar(
    width: u16,
    lead: String,
    progress_data: f64,
    trail: String,
) -> String {
    let mut progress_string = lead.to_owned();

    let progress_bar_width = width - lead.chars().count() as u16 - trail.chars().count() as u16;

    let mut symbol = String::from("|");
    if (0.0..0.5).contains(&progress_data) {
        symbol = symbol.green().to_string();
    } else if (0.5..=0.75).contains(&progress_data) {
        symbol = symbol.yellow().to_string();
    } else {
        symbol = symbol.red().to_string();
    }

    for k in 0..progress_bar_width - 1 {
        //TODO recognize why - 1 ???
        if k < (progress_bar_width as f64 * progress_data) as u16 {
            progress_string = progress_string + &symbol;
        } else {
            progress_string += " ";
        }
    }
    progress_string = progress_string + &trail;
    progress_string
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum LayoutType {
    #[default]
    LayoutCPU,
    LayoutGPU,
}

pub trait Device {
    fn set_position(&mut self, layout_bbox: LayoutBbox);
    fn draw(&mut self, stdout: &mut Stdout);
    fn get_type(&self) -> LayoutType;
}

pub struct LayoutBbox {
    pub top: u16,
    pub left: u16,
    pub width: u16,
    pub height: u16,
}

pub struct LayoutCPU {
    layout_header: String,
    layout_bbox: LayoutBbox,
    layout_device: CpuInfo,
    layout_type: LayoutType,
}

impl LayoutCPU {
    fn new(layout_header: String, layout_bbox: LayoutBbox) -> Self {
        LayoutCPU {
            layout_header: layout_header.bold().to_string(),
            layout_bbox,
            layout_device: CpuInfo::new(),
            layout_type: LayoutType::LayoutCPU,
        }
    }
}

impl Device for LayoutCPU {
    fn set_position(&mut self, layout_bbox: LayoutBbox) {
        self.layout_bbox = layout_bbox;
    }

    fn draw(&mut self, stdout: &mut Stdout) {
        self.layout_device.update();

        execute!(
            stdout,
            MoveTo(self.layout_bbox.left, self.layout_bbox.top),
            Print(&self.layout_header),
        )
        .unwrap();

        // cpu usage
        for i in 0..self.layout_device.cpu_count {
            let cpu_usage = self.layout_device.get_cpu_usage(i);

            let cpu_bar = calculate_progress_bar(
                self.layout_bbox.width,
                format!("{:3}[", i),
                cpu_usage / 100.0,
                format!("{:.2}%]", cpu_usage),
            );
            execute!(
                stdout,
                MoveTo(self.layout_bbox.left, self.layout_bbox.top + 1 + i as u16),
                Print(cpu_bar)
            )
            .unwrap();
        }

        //memory usage
        let ram_usage = self.layout_device.get_ram_usage();

        let ram_bar = calculate_progress_bar(
            self.layout_bbox.width,
            String::from("RAM["),
            ram_usage.0 as f64 / ram_usage.1 as f64,
            format!("{}/{}Mb]", ram_usage.0, ram_usage.1),
        );
        execute!(
            stdout,
            MoveTo(
                self.layout_bbox.left,
                self.layout_bbox.top + 1 + self.layout_device.cpu_count as u16
            ),
            Print(ram_bar)
        )
        .unwrap();

        let swap_usage = self.layout_device.get_swap_usage();
        let swap_bar = calculate_progress_bar(
            self.layout_bbox.width,
            String::from("SWP["),
            swap_usage.0 as f64 / swap_usage.1 as f64,
            format!("{}/{}Mb]", swap_usage.0, swap_usage.1),
        );
        execute!(
            stdout,
            MoveTo(
                self.layout_bbox.left,
                self.layout_bbox.top + 2 + self.layout_device.cpu_count as u16
            ),
            Print(swap_bar)
        )
        .unwrap();
    }

    fn get_type(&self) -> LayoutType {
        self.layout_type
    }
}

pub struct LayoutGpu {
    layout_header: String,
    layout_bbox: LayoutBbox,
    layout_device: GpuAll,
    layout_type: LayoutType,
}

impl LayoutGpu {
    fn new(layout_header: String, layout_bbox: LayoutBbox) -> Self {
        let device = GpuAll::new();
        LayoutGpu {
            layout_header: layout_header.bold().to_string(),
            layout_bbox,
            layout_device: device,
            layout_type: LayoutType::LayoutGPU,
        }
    }
}

impl Device for LayoutGpu {
    fn set_position(&mut self, layout_bbox: LayoutBbox) {
        self.layout_bbox = layout_bbox;
    }

    fn draw(&mut self, stdout: &mut Stdout) {
        self.layout_device.update();

        execute!(
            stdout,
            MoveTo(self.layout_bbox.left, self.layout_bbox.top),
            Print(&self.layout_header),
        )
        .unwrap();

        let mut new_top = self.layout_bbox.top + 1;

        for device_index in 0..self.layout_device.device_count {
            let mut device_info = self.layout_device.get_info(device_index);
            if device_info.len() > self.layout_bbox.width as usize {
                device_info = String::from(&device_info[..self.layout_bbox.width as usize]);
            }

            // calculate memory used progress string
            let memory_data = self.layout_device.get_memory_info(device_index);
            let memory_bar = calculate_progress_bar(
                self.layout_bbox.width,
                String::from("Mem["),
                memory_data.0 / memory_data.1,
                format!("{}/{}Mb]", memory_data.0, memory_data.1),
            );

            // calculate utilization_rate string
            let util_rate = self.layout_device.get_utilization_rate_info(device_index);
            let util_rate_bar = calculate_progress_bar(
                self.layout_bbox.width,
                String::from("GPU["),
                util_rate / 100.0,
                format!("{}%]", util_rate),
            );

            execute!(
                stdout,
                MoveTo(self.layout_bbox.left, new_top),
                Print(device_info),
                MoveTo(self.layout_bbox.left, new_top + 1),
                Print(memory_bar),
                MoveTo(self.layout_bbox.left, new_top + 2),
                Print(util_rate_bar),
            )
            .unwrap();
            new_top += 4;
        }
    }
    fn get_type(&self) -> LayoutType {
        self.layout_type
    }
}

pub struct Ui {
    stdout: Stdout,
    width: u16,
    height: u16,
    grid_width: u16,
    grid_height: u16,
    layouts: Vec<Box<dyn Device>>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            width: 1,
            height: 1,
            grid_height: 1,
            grid_width: 1,
            layouts: Vec::new(),
        }
    }

    pub fn create_layout(
        &mut self,
        layout_header: String,
        layout_bbox: LayoutBbox,
        layout_type: LayoutType,
    ) {
        match layout_type {
            LayoutType::LayoutGPU => {
                let device = LayoutGpu::new(layout_header, layout_bbox);
                self.layouts.push(Box::new(device));
            }
            LayoutType::LayoutCPU => {
                let device = LayoutCPU::new(layout_header, layout_bbox);
                self.layouts.push(Box::new(device));
            }
        };
    }

    pub fn update_all(&mut self) {
        self.clear_screen();

        self.update_screen();

        for layout in &mut self.layouts {
            layout.draw(&mut self.stdout);
        }

        self.show_status_line();
    }

    fn update_screen(&mut self) {
        (self.width, self.height) = terminal::size().expect("Can't get terminal size");

        for device in &mut self.layouts[..] {
            match device.get_type() {
                LayoutType::LayoutCPU => {
                    device.set_position(LayoutBbox {
                        top: 0,
                        left: 0,
                        width: self.width / 2,
                        height: self.height / 2,
                    });
                }
                LayoutType::LayoutGPU => {
                    device.set_position(LayoutBbox {
                        top: 0,
                        left: self.width / 2 + 1,
                        width: self.width / 2,
                        height: self.height / 2,
                    });
                }
            }
        }
    }

    fn clear_screen(&mut self) {
        execute!(&mut self.stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    }

    fn show_status_line(&mut self) {
        execute!(
            self.stdout,
            MoveTo(0, self.height - 1),
            SetForegroundColor(Color::Green),
            Print("Press q for exit...".to_string()),
            ResetColor,
        )
        .unwrap();
    }
}
