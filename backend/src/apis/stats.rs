#[allow(non_snake_case)]
pub mod Stats {
    use std::sync::{Arc, Mutex};

    use crate::Config::*;
    use rocket::{
        get,
        http::Status,
        State,
    };

    #[utoipa::path(
        get,
        context_path = "/stats",
        responses(
            (status = 200, description = "Successfully grabbed media stats")
        )
    )]
    #[get("/media")]
    pub async fn media(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
    ) -> Status {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/stats",
        responses(
            (status = 200, description = "Successfully grabbed user stats")
        )
    )]
    #[get("/user")]
    pub async fn user(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
    ) -> Status {
        todo!()
    }
}