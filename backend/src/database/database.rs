pub mod database {
    use std::path::{PathBuf};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Media {
        pub id: String,
        pub name: String,
        pub extension: String,
        pub data_size: i32,
        pub data_path: PathBuf,
        pub data_compressed: bool,
        pub upload_date: DateTime::<Utc>,
        pub author_id: i32,
        pub private: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct User {
        pub id: String,
        pub username: String,
        pub creation_date: DateTime::<Utc>,
        // Vec string of Media id's
        pub uploads: Vec<String>,
        pub api_key: String,
        pub password: String,
        pub salt: Vec<u8>
    }
}