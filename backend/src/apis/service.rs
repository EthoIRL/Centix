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
    
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct ApiConfig {
        pub content_id_length: i32,
        pub content_name_length: i32,
        pub content_max_size: i32,
        pub allow_content_editing: bool,
        pub allow_custom_tags: bool,
        pub custom_tag_length: i32,
        pub use_invite_keys: bool,
        pub allow_user_registration: bool,
        // In the form of megabytes (100 mb)
        pub user_upload_limit: i32,
        pub domains: Vec<String>
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
    ) -> Result<Json<ApiConfig>, status::Custom<Error>> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Error {
                error: String::from("An internal error on the server's end has occurred")
            }))
        };

        Ok(Json(
            ApiConfig::from(config.clone())
        ))
    }
}