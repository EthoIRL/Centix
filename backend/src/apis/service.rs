#[allow(non_snake_case)]
pub mod Service {
    use std::{sync::{Arc, Mutex}};

    use crate::{Config, Error};
    use rocket::{
        get,
        State,
        serde::json::Json, response::status, http::Status
    };

    /// Grabs config on the Centix instance
    #[utoipa::path(
        get,
        context_path = "/api/services",
        responses(
            (status = 200, description = "Successfully grabbed centix instance's config", body = Config),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/config")]
    pub async fn config(
        config_store: &State<Arc<Mutex<Config>>>,
        _database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<Config>, status::Custom<Error>> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Error {
                error: String::from("An internal error on the server's end has occurred")
            }))
        };

        Ok(Json(
            config.clone()
        ))
    }
}