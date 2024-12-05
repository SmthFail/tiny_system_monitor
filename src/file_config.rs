use::serde::{Deserialize, Serialize};
use::serde_json;
use::std::{env, fs};
use std::path::PathBuf;
use::std::process::exit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDevice {
    #[serde(rename="type")]
    pub device_type: String,
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileConfig {
    pub name: String,
    pub symbol: String,
    pub devices: Vec<FileDevice> 
    
}

impl Default for FileConfig {
    fn default() -> Self {
        FileConfig{
            name: "default".to_string(),
            symbol: "|".to_string(),
            devices: vec![
                FileDevice{
                    device_type: "cpu".to_string(),
                    row: 0,
                    col: 0,
                    width: 1,
                    height: 2
                },
                FileDevice{
                    device_type: "gpu".to_string(),
                    row: 0, 
                    col: 1,
                    width: 2,
                    height: 1
                }
            ]
        }
    }
}

impl FileConfig {
    pub fn new(config_name: String) -> Self {
        if config_name.is_empty() {
            Self::default()
        }
        else {
            let home_dir = if cfg!(target_os="windows") {
                env::var("USERPROFILE").expect("Can't get user directory")
            } else {
                env::var("HOME").expect("Can't get home directory")
            };

            let mut config_path = PathBuf::from(home_dir);
            config_path.push(format!(".config/tsm/{config_name}.json"));

            Self::load_config_from_file(&config_path)
        }
    }

    fn load_config_from_file(path: &PathBuf) -> Self {
        let file_config: FileConfig = match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|err| {
                eprintln!("ERROR: Could not deserialize file with error: {err}");
                exit(1)
            }),
            Err(err) => {
                eprintln!("ERROR: Could not open file in {} with error: {}", path.display(), err);
                exit(1)
            }
        };
        file_config
    }

}
