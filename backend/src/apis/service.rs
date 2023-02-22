#[allow(non_snake_case)]
pub mod Service {
    use std::{sync::{Arc, Mutex}};

    use crate::{Config, Error, config as cfg};
    use rocket::{
        get,
        State,
        serde::json::Json, response::status, http::Status,
        serde::{Serialize, Deserialize}
    };
    use utoipa::ToSchema;

    use git_version::git_version;
    const GIT_VERSION: &str = git_version!();

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct ApiConfig {
        // Media related
        pub media_allow_editing: bool,
        pub media_max_name_length: i32,
        
        // Service related
        pub backend_domains: Vec<String>,

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
        pub user_password_limit: i32

        // TODO: Ignore certain settings if user is an admin
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct Information {
        pub git_version: String
    }

    impl From<cfg::Config> for ApiConfig {
        fn from(config: cfg::Config) -> Self {
            let serialized = serde_json::to_string(&config).unwrap();
            serde_json::from_str(&serialized).unwrap()
        }
    }

    /// Grabs config on the Centix instance
    #[utoipa::path(
        get,
        context_path = "/api/services",
        responses(
            (status = 200, description = "Successfully grabbed centix instance's config", body = ApiConfig),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/config")]
    pub async fn config(
        config_store: &State<Arc<Mutex<Config>>>,
        _database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<ApiConfig>, status::Custom<Json<Error>>> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        Ok(Json(
            ApiConfig::from(config.clone())
        ))
    }

    /// Grabs information about the Centix instance
    #[utoipa::path(
        get,
        context_path = "/api/services",
        responses(
            (status = 200, description = "Successfully grabbed centix instance's information", body = Information),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/information")]
    pub async fn info(
        _config_store: &State<Arc<Mutex<Config>>>,
        _database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<Information>, status::Custom<Json<Error>>> {
        Ok(Json(Information{
            git_version: GIT_VERSION.to_string()
        }))
    }
}