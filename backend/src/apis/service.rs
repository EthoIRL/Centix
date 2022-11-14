#[allow(non_snake_case)]
pub mod Service {
    use std::{sync::{Arc, Mutex}};

    use crate::{Config::*, Error};
    use rocket::{
        get,
        State,
        serde::{Serialize, Deserialize, json::Json}
    };

    use utoipa::{IntoParams, ToSchema};

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct DomainInfo {
        #[schema(example = "List of avaliable domains on this centix instance")]
        domains: Vec<String>
    }

    #[utoipa::path(
        get,
        context_path = "/services",
        responses(
            (status = 200, description = "Successfully grabbed all avaliable domains pointing to centix instance"),
            (status = 500, description = "An internal error on the server's end has occured", body = Error::InternalError)
        )
    )]
    #[get("/domains")]
    pub async fn domains(
        config_store: &State<Arc<Mutex<Config>>>,
        _database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<DomainInfo>, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occured")))
        };

        Ok(Json(DomainInfo {
            domains: config.domains.clone()
        }))
    }
}