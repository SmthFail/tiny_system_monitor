use sysinfo::{System, Networks};
use std::cell::RefCell;
use std::rc::Rc;

use crate::tui::widget::{Widget, Rect, ChildConstraints};
use std::time::Instant;
use crate::tui::container_widget::{Container, Layout, Alignment};
use crate::TextWidget;
use crate::Buffer;

use crate::tui::app_error::AppError;



pub struct NetworkInfo {
    sys: System,
    networks: Networks,
    previous_time: Instant,
    previous_rx: u64,
    previous_tx: u64,
    rate_string: Rc<RefCell<String>>,
    constraints: ChildConstraints,
    ui: Box<dyn Widget>
}

impl NetworkInfo {
    pub fn new(border: bool) -> Self {
        let sys = System::new();
        let networks = Networks::new_with_refreshed_list();
        let previous_time = Instant::now();
        
        let mut previous_tx = 0;
        let mut previous_rx = 0;

        for (_, data) in &networks {
            previous_rx += data.received();
            previous_tx += data.transmitted();
        }

        //calculate constraints
        
        let height = if border {
            Some(4)
        } else {
            Some(2)
        };

        let constraints = ChildConstraints {
            min_width: None,
            max_width: None,
            min_height: height,
            max_height: height
        };

        //create ui
        let mut ui = Container::new(None, None, Layout::Vertical, Alignment::Start, border);
        ui.add_child(Box::new(TextWidget::new_static("Network info")));

        let dumb_string = Self::format_rate_string(0.0, 0.0);
        let rate_string = Rc::new(RefCell::new(dumb_string));
        ui.add_child(Box::new(TextWidget::new_editable(rate_string.clone())));

        NetworkInfo {
            sys,
            networks,
            previous_time,
            previous_rx,
            previous_tx,
            rate_string,
            constraints,
            ui: Box::new(ui)
        }
    }

    fn format_speed(bytes_per_sec: f64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * KB;
        const GB: f64 = 1024.0 * MB;

        if bytes_per_sec >= GB {
            format!("{:.2}GB/s", bytes_per_sec / GB)
        } else if bytes_per_sec >= MB {
            format!("{:.2}MB/s", bytes_per_sec / MB)
        } else if bytes_per_sec >= KB {
            format!("{:.2}KB/s", bytes_per_sec / KB)
        } else {
            format!("{:.2} B/s", bytes_per_sec)
        }
        

    }

    fn format_rate_string(rx: f64, tx: f64) -> String  {
        format!("Rx: {}, Tx: {}", 
            Self::format_speed(rx), 
            Self::format_speed(tx)
            )
    }


    fn update_data(&mut self) {
        self.networks.refresh(true);
        let current_time = Instant::now();
        let duration = current_time.duration_since(self.previous_time).as_secs_f64();

        let mut current_rx = 0;
        let mut current_tx = 0;
        for (_, data) in &self.networks {
            current_rx += data.received();
            current_tx += data.transmitted();
        }

        let rx_rate = current_rx.saturating_sub(self.previous_rx) as f64 / duration;
        let tx_rate = current_rx.saturating_sub(self.previous_rx) as f64 / duration;

        *self.rate_string.borrow_mut() = Self::format_rate_string(rx_rate, tx_rate);

        self.previous_rx = current_rx;
        self.previous_tx = current_tx;
        self.previous_time = current_time;
        
    }
}


impl Widget for NetworkInfo {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        self.ui.render(buf, area)?;
        Ok(())
    }

    fn update(&mut self) -> Result<(), AppError>{
        self.update_data();
        self.ui.update()?;
        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
       self.constraints.clone() // TODO
    }
}


