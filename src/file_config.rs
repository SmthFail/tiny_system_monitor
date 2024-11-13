use::serde::{Deserialize, Serialize};
use::serde_json;
use::std::fs;
use::std::process::exit;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileDevice {
    #[serde(rename="type")]
    pub device_type: String,
    #[serde(rename="topLeft")]
    pub top_left: (u8, u8),
    pub width: u8,
    pub height: u8
}



#[derive(Serialize, Deserialize, Debug)]
pub struct FileConfig {
    #[serde(default="get_default_name")]
    pub name: String,
    pub devices: Vec<FileDevice> 
    
}

fn get_default_name() -> String {
    "default".to_string()
}

impl FileConfig {
    pub fn new(config_name: String) -> Self {
        if config_name.is_empty() {
            // TODO implement with structure default values
            Self{
                name: "default".to_string(),
                devices: vec![
                    FileDevice{
                        device_type: "cpu".to_string(),
                        top_left: (0, 0),
                        width: 1,
                        height: 2
                    },
                    FileDevice{
                        device_type: "gpu".to_string(),
                        top_left: (0, 1),
                        width: 2,
                        height: 1
                    }
                ]
            }
        }
        else {
            let config_path = format!("/home/user/works/git/system_monitor/config/{config_name}.json");
            Self::load_config_from_file(&config_path)
        }
    }

    fn load_config_from_file(path: &str) -> Self {
        let devices: Vec<FileDevice> = match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|err| {
                eprintln!("ERROR: Could not deserialize file with error: {err}");
                exit(1)
            }),
            Err(err) => {
                eprintln!("ERROR: Could not open file in {path} with error: {err}");
                exit(1)
            }
        }; 
        Self{
            name: path.to_string(),
            devices
        } 
    }

}
