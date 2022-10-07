use std::{path::Path, fs::{File, self}, io::BufReader, sync::{Arc, Mutex}};

use rocket::{routes, Build, Rocket};
use utoipa::{OpenApi, openapi};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::Modify;

pub mod apis {
    pub mod media;
    pub mod user;
    pub mod stats;
}

pub mod database {
    pub mod database;
}

use crate::apis::media::Media;
use crate::apis::user::User;
use crate::apis::stats::Stats;

#[allow(non_snake_case)]
pub mod Config {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub content_directory: Option<String>,
        pub content_compression: bool,
        pub content_id_length: i32,
        pub use_invite_keys: bool,
        pub allow_user_registration: bool,
        pub first_user_admin: bool
    }
    
    impl Config {
        pub fn new() -> Config {
            Config { content_directory: None, content_compression: true, content_id_length: 8, use_invite_keys: false, allow_user_registration: true, first_user_admin: true }
        }
    }
}


#[derive(OpenApi)]
    #[openapi(
        paths(
            Media::grab,
            Media::all,
            Media::upload,
            Media::delete,
            User::register,
            User::login,
            User::delete,
            User::update_username,
            User::update_password,
            User::generate_invite,
            Stats::media,
            Stats::user
        ),
        components(
            schemas(Media::Media, Media::UploadParam, Media::ContentType),
            schemas(User::Error)
        ),
        tags(
            (name = "Media", description = "All media management api endpoints."),
            (name = "User", description = "All user management api endpoints."),
            (name = "Stats", description = "All statistical management api endpoints.")
        )
    )]
    struct ApiDoc;

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    env_logger::init();

    let config_path = Path::new("./config.json");
    let mut config: Option<Config::Config> = Option::None;
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
        config = Some(Config::Config::new());
        // Hope it writes to file.
        let _ = fs::write(config_path, serde_json::to_string_pretty(&config).unwrap());
    }

    let database = match sled::open("database") {
        Ok(result) => result,
        Err(error) => panic!("{error}")
    };
    let database_arc = Arc::new(Mutex::new(database));

    let config_arc = Arc::new(Mutex::new(config.unwrap()));

    let doc = &mut ApiDoc::openapi();
    ApiDoc::modify(&ApiDoc, doc);

    rocket::build()
        .manage(config_arc)
        .manage(database_arc)
        .mount(
            "/",
            SwaggerUi::new("/swagger/<_..>").url("/api-doc/openapi.json", doc.to_owned()),
        )
        .mount(
            "/media",
            routes![
                Media::all,
                Media::upload,
                Media::delete
            ]
        )
        .mount(
            "/",
            routes![
                Media::grab
            ]
        )
        .mount(
            "/user", 
            routes![
                User::register,
                User::login,
                User::delete,
                User::update_username,
                User::update_password,
                User::generate_invite
            ]
        )
        .mount(
            "/stats", 
            routes![
                Stats::media,
                Stats::user
            ]
        )
}

impl Modify for ApiDoc {    
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        openapi.info.title = String::from("Centix Backend");
        openapi.info.description = Some(String::from("Centix backend api"));
        openapi.info.license = None;
        openapi.info.version = String::from("V1");
    }
}