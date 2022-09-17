pub mod database {
    use std::path::{PathBuf};

    use chrono::{DateTime, Utc};
    
    struct Media {
        id: String,
        name: String,
        extension: String,
        data_size: i32,
        data_path: PathBuf,
        data_compressed: bool,
        upload_date: DateTime::<Utc>,
        author_id: i32,
        private: bool,
    }

    struct User {
        id: i32,
        username: String,
        creation_date: DateTime::<Utc>,
        // Vec string of Media id's
        uploads: Vec<String>,
        api_key: String
    }
}