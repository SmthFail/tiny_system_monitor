mod app_config;
mod file_config;
mod utils;      

mod devices;
use crate::devices::{cpu_info, gpu_info, network_info};

mod tui;
use tui::app::App;
use tui::container_widget::{Container, Alignment, Layout};
use tui::text_widget::TextWidget;
use crate::tui::buffer::Buffer;


use utils::version_checker::get_version;

fn print_usage_message() {
    println!("Usage: ");
    println!("tsm  [Options]");
    println!("Options:");
    println!("  <config_name>   Read config with the given name. Config must be placed in ~/.config/tsm/<config_name>.json");
    println!("  -h, --help      Print help message")
}



fn main() {
    
    let version = get_version();
    
    let mut app = App::new();

    // main container 
    let mut h_container = Container::new(None, None, Layout::Horizontal, Alignment::Start, false);
    h_container.add_child(Box::new(cpu_info::CpuInfo::new()));

    let mut v_container = Container::new(None,None, Layout::Vertical, Alignment::Start, false);
    v_container.add_child(Box::new(gpu_info::GpuInfo::new()));
    v_container.add_child(Box::new(network_info::NetworkInfo::new(true)));
    h_container.add_child(Box::new(v_container));

    app.add_child(Box::new(h_container));

    let status_string = format!("q: exit, ver: {:?}", version);
    app.add_child(Box::new(TextWidget::new(&status_string)));
    app.run();
  }
