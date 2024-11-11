use::std::{env, fs};
use::std::process::exit;
use::serde::{Deserialize, Serialize};
use::serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Device {
    #[serde(rename="type")]
    device_type: String,
    #[serde(rename="topLeft")]
    top_left: (u8, u8),
    width: u8,
    height: u8
}



#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    #[serde(default="get_default_name")]
    name: String,
    devices: Vec<Device> 
    
}

fn get_default_name() -> String {
    "default".to_string()
}

impl AppConfig {
    pub fn new(mut config_name: String) -> Self {
        if config_name.is_empty() {
            config_name = "default".to_string()
            // TODO get default config from memory not file
        }
        else {
            // TODO get config file here
        }
        let config_path = format!("/home/smthfail/work/git/system_monitor/config/{config_name}.json");
        let devices: Vec<Device> = match fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|err| {
                eprintln!("ERROR: Could not deserialize file with error: {err}");
                exit(1)
            }),
            Err(err) => {
                eprintln!("ERROR: Could not open file in {config_path} with error: {err}");
                exit(1)
            }
        }; 
        AppConfig{
            name: config_name,
            devices
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
  
    let config = match args.len() - 1 {
        0 => AppConfig::new(String::new()),
        1 => AppConfig::new(args[1].clone()),
        _ => { 
            eprint!("ERROR: Invalid number of arguments");
            return;
        }
    };

    println!("{:?}", &config)
}
