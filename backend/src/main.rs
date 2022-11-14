use std::{
    path::Path,
    fs::{File, self},
    io::BufReader,
    sync::{Arc, Mutex}
};

use rocket::{
    routes, Build,
    Rocket,
    serde::Serialize,
    Responder
};

use utoipa::{
    OpenApi,
    openapi
};

use utoipa::{
    Modify,
    ToSchema
};

use utoipa_swagger_ui::SwaggerUi;

pub mod apis {
    pub mod media;
    pub mod user;
    pub mod stats;
    pub mod service;
}

pub mod database {
    pub mod database;
}

use crate::apis::media::Media;
use crate::apis::user::User;
use crate::apis::stats::Stats;
use crate::apis::service::Service;

#[allow(non_snake_case)]
pub mod Config {
    use serde::{Deserialize, Serialize};

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
        pub use_invite_keys: bool,
        pub allow_user_registration: bool,
        pub first_user_admin: bool,
        pub store_compressed: bool,
        pub domains: Vec<String>
    }

    impl Default for Config {
        fn default() -> Self {
            // content_compression: true, content_compression_target: 75
            Config { content_directory: None, content_id_length: 8, content_name_length: 32, content_max_size: 24, use_invite_keys: false, allow_user_registration: true, first_user_admin: true, store_compressed: true, domains: Vec::new() }
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        Media::info,
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
        User::invite_info,
        Stats::media,
        Stats::user,
        Service::domains
    ),
    components(
        schemas(Media::Media, Media::UploadParam, Media::ContentType, Media::ContentInfo),
        schemas(Stats::MediaStats, Stats::UserStats),
        schemas(User::InviteInfo),
        schemas(Service::DomainInfo),
        schemas(Error)
    ),
    tags(
        (name = "Media", description = "All media management related api endpoints."),
        (name = "User", description = "All user management related api endpoints."),
        (name = "Stats", description = "All statistical management related api endpoints."),
        (name = "Service", description = "All service related api endpoints.")
    )
)]
struct ApiDoc;

// TODO: Option's don't work in status code responses
#[derive(Serialize, ToSchema, Responder, Debug)]
pub enum Error {
    #[response(status = 400)]
    BadRequest(String),

    #[response(status = 401)]
    Unauthorized(String),

    #[response(status = 403)]
    Forbidden(String),

    #[response(status = 405)]
    NotAllowed(String),

    #[response(status = 500)]
    InternalError(String)
}

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
        config = Some(Config::Config::default());
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
                Media::info,
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
                User::generate_invite,
                User::invite_info
            ]
        )
        .mount(
            "/stats", 
            routes![
                Stats::media,
                Stats::user
            ]
        )
        .mount(
            "/services", 
            routes![
                Service::domains
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