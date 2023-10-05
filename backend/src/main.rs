// TODO: Implement backend wide runtime tests (https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
use std::sync::{Arc, Mutex};

use log::warn;
use rocket::{
    serde::{Serialize, Deserialize},
    Responder,
    routes,
    data::{Limits, ByteUnit},
    config::TlsConfig
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
use rocket_analytics::{self, Analytics};

// TODO: Remove this and use traditional mod.rs files where applicable
pub mod apis {
    pub mod media;
    pub mod user;
    pub mod stats;
    pub mod service;
}

pub mod database {
    pub mod database;

    pub mod database_utils;
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
        Service::config,
        Service::info
    ),
    components(
        schemas(Media::Media, Media::ContentType, Media::ContentInfo, Media::ContentFound, Media::ContentTags,
            Media::SearchQuery, Media::UploadMedia, Media::DeleteMedia, Media::EditMedia),
        schemas(Stats::MediaStats, Stats::UserStats),
        schemas(User::InviteInfo, User::UserInvite, User::UserApiKey, User::UserList, User::UserInfo, User::UserCredentials, User::UserRegistration,
            User::UserUpdateUsername, User::UserUpdatePassword, User::InviteInfoRequest),
        schemas(Error, Service::ApiConfig, Service::Information)
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

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
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

    let limits = Limits::new()
        .limit("json", ByteUnit::max_value());

    let mut figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("limits", limits));

    let tls_cert_path = &config_arc.lock().unwrap().backend_tls_cert_path.clone();
    let tls_key_path = &config_arc.lock().unwrap().backend_tls_key_path.clone();

    if tls_cert_path.is_some() && tls_key_path.is_some() {
        let cert_path = tls_cert_path.clone().unwrap();
        let key_path = tls_key_path.clone().unwrap();

        if cert_path.is_empty() || key_path.is_empty() {
            warn!("[TLS] Certificate or Key path is empty!")
        } else {
            let tls_config = TlsConfig::from_paths(cert_path, key_path)
                .with_preferred_server_cipher_order(false);
            figment = figment.merge(("tls", tls_config));
        }
    } else {
        warn!("[TLS] Certificate or Key path unset!")
    }

    let key = &config_arc.lock().unwrap().backend_analytics_key.clone();

    let mut rocket_builder = rocket::build()
        .configure(figment)
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
                    Service::config,
                    Service::info
                ]
        );

    if let Some(analytics_key) = key {
        rocket_builder = rocket_builder.attach(Analytics::new(analytics_key.to_string()));
    }

    rocket_builder.launch().await?;

    Ok(())
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
            for domain in cfg.backend_domains {
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