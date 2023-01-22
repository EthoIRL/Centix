use std::{
    sync::{Arc, Mutex}
};

use rocket::{
    serde::{Serialize, Deserialize},
    Responder,
    routes, Build,
    Rocket
};


use utoipa::{
    OpenApi,
    openapi::{self, Server}
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

pub mod config;

use crate::apis::media::Media;
use crate::apis::user::User;
use crate::apis::stats::Stats;
use crate::apis::service::Service;
use crate::config::Config;

#[derive(OpenApi)]
#[openapi(
    paths(
        Media::info,
        Media::download,
        Media::search,
        Media::upload,
        Media::delete,
        Media::edit,
        Media::tags,
        User::register,
        User::login,
        User::delete,
        User::update_username,
        User::update_password,
        User::generate_invite,
        User::invite_info,
        User::list,
        User::info,
        Stats::media,
        Stats::user,
        Service::config
    ),
    components(
        schemas(Media::Media, Media::UploadParam, Media::ContentType, Media::ContentInfo, Media::ContentFound, Media::ContentId, Media::ContentTags),
        schemas(Stats::MediaStats, Stats::UserStats),
        schemas(User::InviteInfo, User::UserInvite, User::UserKey, User::UserList, User::UserInfo),
        schemas(Error, Config)
    ),
    tags(
        (name = "Media", description = "All media management related api endpoints."),
        (name = "User", description = "All user management related api endpoints."),
        (name = "Stats", description = "All statistical management related api endpoints."),
        (name = "Service", description = "All service related api endpoints."),
        (name = "Admin", description = "All admin related api endpoints.")
    )
)]
struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema, Responder, Debug)]
pub struct Error {
    pub error: String,
}

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    // env_logger::init();

    let config = config::grab_config();

    if config.is_none() {
        panic!("Config couldn't be generated or grabbed!");
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
            "/api/media",
            routes![
                Media::info,
                Media::download,
                Media::search,
                Media::upload,
                Media::delete,
                Media::edit,
                Media::tags
            ]
        )
        .mount(
            "/api/user", 
            routes![
                User::register,
                User::login,
                User::delete,
                User::update_username,
                User::update_password,
                User::generate_invite,
                User::invite_info,
                User::list,
                User::info
            ]
        )
        .mount(
            "/api/stats", 
            routes![
                Stats::media,
                Stats::user
            ]
        )
        .mount(
            "/api/services", 
            routes![
                Service::config
            ]
        )
}

impl Modify for ApiDoc {    
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        openapi.info.title = String::from("Centix Backend");
        openapi.info.description = Some(String::from("Centix backend api"));
        openapi.info.license = None;
        openapi.info.version = String::from("V1");
        
        let config = config::grab_config();
        let mut domain_servers: Vec<Server> = Vec::new();

        if let Some(cfg) = config {
            for domain in cfg.domains {
                let mut server = Server::new("/");
                server.description = Some(domain);

                domain_servers.push(server);
            }
        }
        
        if !domain_servers.is_empty() {
            openapi.servers = Some(domain_servers);
        }
    }
}