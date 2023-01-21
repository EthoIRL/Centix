#[allow(non_snake_case)]
pub mod Service {
    use std::{sync::{Arc, Mutex}};

    use crate::{Error, Config};
    use rocket::{
        get,
        State,
        serde::json::Json
    };

    /// Grabs config on the Centix instance
    #[utoipa::path(
        get,
        context_path = "/api/services",
        responses(
            (status = 200, description = "Successfully grabbed centix instance's config"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error::InternalError)
        )
    )]
    #[get("/config")]
    pub async fn config(
        config_store: &State<Arc<Mutex<Config>>>,
        _database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<Config>, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        Ok(Json(
            config.clone()
        ))
    }
}