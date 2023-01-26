#[allow(non_snake_case)]
pub mod Stats {
    use std::sync::{Arc, Mutex};

    use crate::{Config, Error};
    use rocket::{
        get,
        State, serde::json::Json, response::status, http::Status
    };

    use serde::{Serialize, Deserialize};
    use utoipa::{IntoParams, ToSchema};

    use crate::database::database::Media;

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct MediaStats {
        /// Total uploads on the instance
        media_count: i32,
        /// Total byte size of all uploads
        media_storage_usage: i32
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserStats {
        /// Total user accounts
        user_count: i32
    }

    /// Grabs all media related stats
    /// such as total media count and storage usage regarding media.
    #[utoipa::path(
        get,
        context_path = "/api/stats",
        responses(
            (status = 200, description = "Successfully grabbed media stats", body = MediaStats),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/media")]
    pub async fn media(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<MediaStats>, status::Custom<Json<Error>>> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let media_count: Vec<i32> = media_database.iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| {
                let result: Media = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                    Ok(result) => result,
                    Err(_) => return None
                };
                Some(result)
            })
            .map(|media| media.data_size)
            .collect();
        
        Ok(Json(MediaStats {
            media_count: media_count.len() as i32,
            media_storage_usage: media_count.iter().sum()
        }))
    }

    /// Grabs all user related stats
    /// such as total user accounts.
    #[utoipa::path(
        get,
        context_path = "/api/stats",
        responses(
            (status = 200, description = "Successfully grabbed user stats", body = UserStats),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/user")]
    pub async fn user(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<UserStats>, status::Custom<Json<Error>>> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let users: i32 = user_database.iter()
            .filter_map(|item| item.ok())
            .count() as i32;
        
        Ok(Json(UserStats {
            user_count: users
        }))
    }
}