use std::{
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
        User::list,
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
        (name = "Service", description = "All service related api endpoints."),
        (name = "Admin", description = "All admin related api endpoints.")
    )
)]
struct ApiDoc;

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
                User::invite_info,
                User::list
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