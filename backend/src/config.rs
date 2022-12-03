use serde::{Deserialize, Serialize};
use std::{io::BufReader, path::Path, fs::{File, self}};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub content_directory: Option<String>,
    pub content_id_length: i32,
    pub content_name_length: i32,
    // TODO: -->
    // pub content_compression: bool,
    // // Eg.. 80 = 80% of the original size, 60% of the original size
    // pub content_compression_target: i32,
    // // In the form of mb's 1 = 1mb
    pub content_max_size: i32,
    // TODO: ->
    pub allow_content_editing: bool,
    pub use_invite_keys: bool,
    pub allow_user_registration: bool,
    pub first_user_admin: bool,
    pub store_compressed: bool,
    pub domains: Vec<String>,
    pub tags: Vec<String>
}

impl Default for Config {
    fn default() -> Self {
        // content_compression: true, content_compression_target: 75
        Config { 
            content_directory: None,
            content_id_length: 8,
            content_name_length: 32,
            content_max_size: 24,
            allow_content_editing: true,
            use_invite_keys: false,
            allow_user_registration: true,
            first_user_admin: true,
            store_compressed: true,
            domains: Vec::new(),
            tags: Vec::new()
        }
    }
}

pub fn grab_config() -> Option<Config> {
    let config_path = Path::new("./config.json");
    let mut config: Option<Config> = Option::None;
        
    if config_path.exists() {
        let file = match File::open(config_path) {
            Ok(result) => Some(result),
            Err(_) => None
        };
        if let Some(file) = file {
            let reader = BufReader::new(file);
            config = match serde_json::from_reader(reader) {
                Ok(result) => result,
                Err(_) => None
            }
        } 
    }
        
    if config.is_none() {
        config = Some(Config::default());
    
        let pretty_config: Option<String> = match serde_json::to_string_pretty(&config) {
            Ok(result) => Some(result),
            Err(_) => None
        };

        if let Some(pretty_cfg) = pretty_config {
            if fs::write(config_path, pretty_cfg).is_err() {
                return None
            };
        }
    }
    
    config
}