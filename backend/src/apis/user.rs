#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::Config::*;
    use rocket::{
        get,
        http::Status,
        State,
    };

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully registered account")
        )
    )]
    #[get("/register?<username>&<password>")]
    pub async fn register(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Status {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account")
        )
    )]
    #[get("/login?<username>&<password>")]
    pub async fn login(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> String {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account")
        )
    )]
    #[get("/delete?<username>&<password>")]
    pub async fn delete(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Status {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account")
        )
    )]
    #[get("/update/username?<username>&<password>&<new_username>")]
    pub async fn update_username(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        new_username: String
    ) -> Status {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account")
        )
    )]
    #[get("/update/password?<username>&<password>&<new_password>")]
    pub async fn update_password(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        new_password: String
    ) -> Status {
        todo!()
    }
}
