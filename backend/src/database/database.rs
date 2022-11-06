use std::path::{PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::apis::media::Media::ContentType;
    
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Media {
     // Main key
    pub id: String,
    pub name: String,
    pub extension: String,
    pub data_type: ContentType,
    pub data_size: i32,
    pub data_path: PathBuf,
    pub data_compressed: bool,
    pub upload_date: DateTime::<Utc>,
    pub author_username: String,
    pub private: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    // Main key
    pub username: String,
    pub creation_date: DateTime::<Utc>,
    // Vec string of Media id's
    pub uploads: Vec<String>,
    pub api_key: String,
    pub password: String,
    pub admin: bool,
    pub invite_key: Option<String>
}

// TODO: Single use or multi use?
// TODO: Expirtation date?
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Invite {
    // Main key
    pub key: String,
    pub invitee_username: Option<String>,
    pub invitee_date: Option<DateTime::<Utc>>,

    pub creation_date: DateTime::<Utc>,
    pub creator_username: String,
    pub used: bool
}