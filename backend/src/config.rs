use serde::{Deserialize, Serialize};
use std::{io::BufReader, path::Path, fs::{File, self}};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    // Media related
    pub media_allow_editing: bool,
    pub media_max_name_length: i32,
    pub media_dynamic_id_length: i32,
    
    // Service related
    pub backend_store_compressed: bool,
    pub backend_domains: Vec<String>,
    pub backend_media_directory: Option<String>,
    pub backend_analytics_key: Option<String>,

    // Media tags
    pub tags_default: Vec<String>,
    pub tags_allow_custom: bool,
    pub tags_max_name_length: i32,

    // Registration related
    pub registration_allow: bool,
    pub registration_use_invite_keys: bool,

    // User related
    // 60 individual content pieces (Ignore if admin, or if value = 0)
    pub user_upload_limit: i32,
    // 12 mb per content piece (Ignore if admin, or if value = 0)
    pub user_upload_size_limit: i32, 
    // 120 mb total per account (Ignore if admin. or if value = 0)
    pub user_total_upload_size_limit: i32, 

    pub user_username_limit: i32,
    pub user_password_limit: i32,
    pub user_first_admin: bool

    // TODO: Stats -> MediaStats, Allow user to specify whether to show unlisted upload count
}

impl Default for Config {
    fn default() -> Self {
        Config {
            media_allow_editing: true,
            media_max_name_length: 32, 
            media_dynamic_id_length: 4, // Maybe go to 6
            
            backend_store_compressed: true,
            backend_domains: Vec::new(),
            backend_media_directory: None,
            backend_analytics_key: None,
            // TODO: Replace this with a "good" default list of tags
            tags_default: vec![String::from("funny"), String::from("meme"), String::from("nsfw"), String::from("clip")],
            tags_allow_custom: false,
            tags_max_name_length: 16,
            registration_allow: true,
            registration_use_invite_keys: false,
            user_upload_limit: 60,
            user_upload_size_limit: 12,
            user_total_upload_size_limit: 120,
            user_username_limit: 24,
            user_password_limit: 128,
            user_first_admin: true
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