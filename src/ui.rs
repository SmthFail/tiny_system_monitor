use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::{execute, terminal};

use std::io::{stdout, Stdout};

use crate::gpu_info;
use gpu_info::GpuAll;

use crate::cpu_info;
use cpu_info::CpuInfo;

use crate::DeviceTile;

extern crate unicode_width;
use unicode_width::UnicodeWidthStr;


pub fn calculate_progress_bar(
    width: u16,
    lead: String,
    progress_data: f64,
    trail: String,
    symbol: &mut String
) -> String {
    let mut progress_string = lead.to_owned();
    let mut symbol = symbol;//String::from("|"); //  🐱  TODO pass symbol from app config
    
    let symbol_width: usize = symbol.width();

    
    let progress_bar_width = width  
        - lead.as_str().width() as u16 
        - trail.as_str().width() as u16;
        
    
    //progress_bar_width = progress_bar_width / symbol_width as u16;
    
    if symbol == "|" {
        if (0.0..0.5).contains(&progress_data) {
           symbol.clone().green().to_string();
        } else if (0.5..=0.75).contains(&progress_data) {
           symbol.clone().yellow().to_string();
        } else {
           symbol.clone().red().to_string();
        }
    } 

    
    let load_width = (progress_bar_width as f64 * progress_data) as usize;
   
    

    for _ in 0..load_width.div_ceil(symbol_width) {
        progress_string = progress_string + &symbol;
    }
    
    
    
    if progress_bar_width as usize - load_width != 0 {
    	let empty_width = progress_bar_width as usize - load_width.div_ceil(symbol_width) * symbol_width - 1;
    	progress_string += &" ".repeat(empty_width);
    }; 
        
    progress_string += &trail;
    progress_string
}

