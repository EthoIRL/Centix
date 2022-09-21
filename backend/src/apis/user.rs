#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::{Config::*, database::database::database::User};
    use rocket::{
        get,
        http::Status,
        State, Responder
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    use pbkdf2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString
        },
        Pbkdf2
    };

    use rand::distributions::{Alphanumeric, DistString};

    #[derive(Serialize, ToSchema, Responder)]
    pub enum Error {
        #[response(status = 401)]
        Unauthorized(String),

        #[response(status = 403)]
        Forbidden(String),

        #[response(status = 405)]
        NotAllowed(String),

        #[response(status = 500)]
        InternalError(Option<String>)
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully registered account"),
            (status = 403, description = "An internal issue has occured when attemping to register an account", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/register?<username>&<password>")]
    pub async fn register(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<Status, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        if !config.allow_user_registration {
            return Err(Error::NotAllowed(String::from("User registration is disabled on this server!")))
        }

        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };
        
        let users: Vec<User> = user_database.iter().map(|item| 
            serde_json::from_str(&String::from_utf8_lossy(&item.unwrap().1.to_vec())).unwrap())
            .collect::<Vec<_>>();
        
        if users.iter().any(|user| user.username == username) {
            return Err(Error::Forbidden(String::from("Username is already in use!")))
        }
        
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = match Pbkdf2.hash_password(password.as_bytes(), &salt) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        }.to_string();
        
        let user = User {
            username: username,
            creation_date: chrono::offset::Utc::now(),
            password: password_hash,
            uploads: Vec::new(),
            api_key: Alphanumeric.sample_string(&mut rand::thread_rng(), 48)
        };

        let user_vec = match serde_json::to_vec(&user) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        match user_database.insert(user.username, user_vec) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        match user_database.flush() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to add user to database"))))
        };
        Ok(Status::Ok)
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account"),
            (status = 403, description = "An internal issue has occured when attemping to register an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/login?<username>&<password>")]
    pub async fn login(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<String, Error> {
        let user_vec = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(user_tree) => {
                        match user_tree.get(&username) {
                            Ok(vec) => {
                                match vec {
                                    Some(result) => result,
                                    None => return Err(Error::InternalError(None))
                                }
                            },
                            Err(_) => return Err(Error::Forbidden(String::from("Password or username is not correct")))
                        }
                    },
                    Err(_) => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec.to_vec())) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        println!("{:#?}", user);

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                return Ok(user.api_key)
            },
            Err(_) => return Err(Error::Forbidden(String::from("Password or username is not correct")))
        };
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
