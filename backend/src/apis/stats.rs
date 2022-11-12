#[allow(non_snake_case)]
pub mod Stats {
    use std::sync::{Arc, Mutex};

    use crate::{Config::*, Error};
    use rocket::{
        get,
        State, serde::json::Json
    };

    use serde::{Serialize, Deserialize};
    use utoipa::{IntoParams, ToSchema};

    use crate::database::database::Media;

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct MediaStats {
        #[schema(example = "Total media uploaded to the service")]
        media_count: i32,
        #[schema(example = "Total byte size of all media uploaded")]
        media_storage_usage: i32
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserStats {
        #[schema(example = "Total user accounts")]
        user_count: i32
    }

    #[utoipa::path(
        get,
        context_path = "/stats",
        responses(
            (status = 200, description = "Successfully grabbed media stats"),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/media")]
    pub async fn media(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<MediaStats>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
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

    #[utoipa::path(
        get,
        context_path = "/stats",
        responses(
            (status = 200, description = "Successfully grabbed user stats"),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/user")]
    pub async fn user(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<UserStats>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let users: i32 = user_database.iter()
            .filter_map(|item| item.ok())
            .count() as i32;
        
        Ok(Json(UserStats {
            user_count: users
        }))
    }
}