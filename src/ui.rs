use crossterm::style::Stylize;
extern crate unicode_width;
use unicode_width::UnicodeWidthStr;


pub fn calculate_progress_bar(
    width: u16,
    lead: &str,
    progress_data: f64,
    trail: &str,
    symbol: &str 
) -> String {
    let mut progress_string = lead.to_string();
    let progress_bar_width = width
        .saturating_sub(lead.width() as u16)
        .saturating_sub(trail.width() as u16);
 
    let colored_symbol = if (0.0..0.5).contains(&progress_data) {
        symbol.green().to_string()
    } else if (0.5..=0.75).contains(&progress_data) {
        symbol.yellow().to_string()
    } else {
        symbol.red().to_string()
    };
   
    let symbol_width: usize = symbol.width();
    let load_width = (progress_bar_width as f64 * progress_data).floor() as usize;
    let full_width = load_width / symbol_width;
    let remaining_width = progress_bar_width as usize - full_width * symbol_width;


    for _ in 0..full_width {
        progress_string.push_str(&colored_symbol);
    }
    
    if remaining_width > 0 {
        progress_string.push_str(&" ".repeat(remaining_width));
    }
       
    progress_string.push_str(trail);
    progress_string
}

