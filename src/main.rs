mod utils;
mod devices;
use crate::devices::{cpu_info, gpu_info, network_info};

mod tui;
use tui::app::App;
use tui::container_widget::{Container, Alignment, Layout};
use tui::text_widget::TextWidget;
use tui::widgets::box_widget::BoxWidget;
use crate::tui::buffer::Buffer;
use utils::version_checker::get_version;


// TODO move it to separate module
use::std::env;


use crate::tui::widget::Widget;

fn debug_draw_childs(root: &dyn Widget, indent: usize) {
        use std::io::{self, Write};
        let mut stdout = io::stdout();


        // print current widget
        let _ = writeln!(
            stdout,
            "{:indent$}{} {:?}",
            "",
            root.display_name(),
            root.get_constraints(),
            indent = indent * 2         
        );

        // recursevly go through children
        for child in root.get_children() {
            debug_draw_childs(child.as_ref(), indent + 1);
        }
    }


fn main() {
    let args: Vec<String> = env::args().collect(); 


    let mut app = App::new().unwrap_or_else(|e| {
        eprintln!("Init app error: {}", e.message);
        std::process::exit(1)
    });

    // main container 
    let mut h_container = Container::new(None, None, Layout::Horizontal, Alignment::Start);
    h_container.add_child(
        BoxWidget::new()
            .border(true)
            .name(String::from("CPU"))
            .child(cpu_info::CpuInfo::new())
    );

    let mut v_container = Container::new(None,None, Layout::Vertical, Alignment::Start);
    use tui::widgets::error_wrappers::with_error_handling;
    v_container.add_child(
        BoxWidget::new()
            .border(true)
            .name(String::from("GPU"))
            .child(with_error_handling(gpu_info::GpuInfo::new()))
    );

    let network_widget = BoxWidget::new()
            .border(true)
            .name(String::from("Network info"))
            .child(network_info::NetworkInfo::new()
    );
    v_container.add_child(network_widget);
    h_container.add_child(v_container);

    app.add_child(h_container);
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--tree" => {
                println!("Not implemented with vnode tree");
                //debug_draw_childs(app., 0);
                return;
            },
            _ => {
                eprintln!("[ERROR] Unknown argument");
                std::process::exit(1); 
            }
        };
    };
    
    if let Err(e) = app.run() {
        eprintln!("App exited with error: {}", e.message);
        std::process::exit(1);
    };
}
