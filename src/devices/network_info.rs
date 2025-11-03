use sysinfo::Networks;
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use std::time::Instant;
use crate::TextWidget;
use crate::Buffer;

use crate::tui::{
    app_error::AppError,
    cell_types::CellString
};



pub struct NetworkInfo {
    networks: Networks,
    previous_time: Instant,
    previous_rx: u64,
    previous_tx: u64,
    rate_string: CellString,
    constraints: ChildConstraints,
    ui: Box<dyn Widget>
}

impl NetworkInfo {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        let previous_time = Instant::now();
        
        let mut previous_tx = 0;
        let mut previous_rx = 0;

        for (_, data) in &networks {
            previous_rx += data.received();
            previous_tx += data.transmitted();
        }

        //calculate constraints
        
        let constraints = ChildConstraints {
            min_width: None,
            max_width: None,
            min_height: Some(1),
            max_height: Some(1)
        };

        //create ui

        let dumb_string = Self::format_rate_string(0.0, 0.0);
        let mut rate_string = CellString::new();
        rate_string.update(dumb_string);
        
        let ui = Box::new(TextWidget::new_editable(rate_string.clone()));

        NetworkInfo {
            networks,
            previous_time,
            previous_rx,
            previous_tx,
            rate_string,
            constraints,
            ui
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

        self.rate_string.update(Self::format_rate_string(rx_rate, tx_rate));

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


