mod utils;
mod devices;
use crate::devices::{cpu_info, gpu_info, network_info};

mod tui;
use tui::app::App;
use tui::container_widget::{Container, Alignment, Layout};
use tui::text_widget::TextWidget;
use crate::tui::buffer::Buffer;
use utils::version_checker::get_version;

fn main() {
    
    let mut app = App::new().unwrap_or_else(|e| {
        eprintln!("Init app error: {}", e.message);
        std::process::exit(1)
    });

    // main container 
    let mut h_container = Container::new(None, None, Layout::Horizontal, Alignment::Start, false);
    h_container.add_child(Box::new(cpu_info::CpuInfo::new()));

    let mut v_container = Container::new(None,None, Layout::Vertical, Alignment::Start, false);
    v_container.add_child(Box::new(gpu_info::GpuInfo::new()));
    v_container.add_child(Box::new(network_info::NetworkInfo::new(true)));
    h_container.add_child(Box::new(v_container));

    app.add_child(Box::new(h_container));

    
    if let Err(e) = app.run() {
        eprintln!("App exited with error: {}", e.message);
        std::process::exit(1);
    };
}
