#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::{Config::*, database::database::database::{User, Invite}};
    use rocket::{
        get,
        http::Status,
        State, Responder
    };
    use serde::{Deserialize, Serialize};
    use sled::IVec;
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
        #[response(status = 400)]
        BadRequest(Option<String>),

        #[response(status = 401)]
        Unauthorized(String),

        #[response(status = 403)]
        Forbidden(Option<String>),

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
            (status = 401, description = "An ", body = Error),
            (status = 403, description = "An internal issue has occured when attemping to register an account", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/register?<username>&<password>&<invite_key>")]
    pub async fn register(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        invite_key: Option<String>
    ) -> Result<Status, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        if !config.allow_user_registration {
            return Err(Error::NotAllowed(String::from("User registration is disabled on this server!")))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => {
                result
            },
            Err(_) => return Err(Error::InternalError(None))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        let users: Vec<User> = user_database.iter().map(|item|
            serde_json::from_str(&String::from_utf8_lossy(&item.unwrap().1.to_vec())).unwrap())
            .collect::<Vec<_>>();

        let mut invite: Option<Invite> = None;

        if config.use_invite_keys && !users.is_empty() {
            let optional_invite = match invite_key.clone() {
                Some(key) => {
                    match invite_database.get(key) {
                        Ok(result) => result,
                        Err(_) => return Err(Error::Unauthorized(String::from("Invitation key does not exist in the database!")))
                    }
                },
                None => {
                    return Err(Error::Unauthorized(String::from("An invitation key is required to register!")))
                }
            };

            invite = match optional_invite {
                Some(invite_vec) => {
                    match serde_json::from_str(&String::from_utf8_lossy(&invite_vec.to_vec())) {
                        Ok(result) => Some(result),
                        Err(_) => return Err(Error::InternalError(None))
                    }
                },
                None => return Err(Error::InternalError(None))
            };
        }
        
        if users.iter().any(|user| user.username == username) {
            return Err(Error::Forbidden(Some(String::from("Username is already in use!"))))
        }
        
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = match Pbkdf2.hash_password(password.as_bytes(), &salt) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        }.to_string();
        
        let user = User {
            username: username.clone(),
            creation_date: chrono::offset::Utc::now(),
            password: password_hash,
            uploads: Vec::new(),
            api_key: Alphanumeric.sample_string(&mut rand::thread_rng(), 48),
            admin: users.is_empty() && config.first_user_admin,
            invite_key: {
                if config.use_invite_keys && !users.is_empty() {
                    invite_key
                } else {
                    None
                }
            }
        };
        
        println!("{:#?}", user);

        let user_vec = match serde_json::to_vec(&user) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        match user_database.insert(user.username, user_vec) { 
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        match user_database.flush() {
            Ok(result) => {
                if let Some(mut invite) = invite {
                    invite = Invite {
                        invitee_username: Some(username.clone()),
                        invitee_date: Some(chrono::offset::Utc::now()),
                        used: true,
                        ..invite
                    };

                    match invite_database.update_and_fetch(&invite.key, |_| {
                        Some(IVec::from(serde_json::to_vec(&invite).unwrap()))
                    }) {
                        Ok(_) => {
                            if let Err(_) = invite_database.flush() {
                                return Err(Error::InternalError(Some(String::from("Failed to update backend database"))))
                            };
                        },
                        Err(_) => return Err(Error::InternalError(None))
                    }

                }
                result
            },
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
        _config_store: &State<Arc<Mutex<Config>>>,
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
                            Err(_) => return Err(Error::Forbidden(Some(String::from("Password or username is not correct"))))
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

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                return Ok(user.api_key)
            },
            Err(_) => return Err(Error::Forbidden(Some(String::from("Password or username is not correct"))))
        };
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully deleted account"),
            (status = 403, description = "An authentication issue has occured when attemping to delete an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/delete?<username>&<password>")]
    pub async fn delete(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<Status, Error> {
        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let user_vec = match user_database.get(&username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Couldn't find user associated with username"))))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec.to_vec())) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                match user_database.remove(username) {
                    Ok(_) => {
                        return Ok(Status::Ok)
                    },
                    Err(_) => return Err(Error::InternalError(Some(String::from("Failed delete user account"))))
                };
            },
            Err(_) => return Err(Error::Forbidden(None))
        };
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account username"),
            (status = 400, description = "The Client has sent a badly formed request", body = Error),
            (status = 403, description = "An authentication issue has occured when attemping to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/update/username?<username>&<password>&<newname>")]
    pub async fn update_username(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        newname: String
    ) -> Result<Status, Error> {
        if username == newname {
            return Err(Error::BadRequest(None))
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

        let user_vec = match user_database.get(username) {
            Ok(result) => {
                match result {
                    Some(result) => {
                        result
                    },
                    None => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Couldn't find user associated with username"))))
        };

        let mut user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec.to_vec())) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                match user_database.remove(&user.username) {
                    Ok(_) => {
                        let users: Vec<User> = user_database.iter().map(|item| 
                            serde_json::from_str(&String::from_utf8_lossy(&item.unwrap().1.to_vec())).unwrap())
                            .collect::<Vec<_>>();
                        
                        if users.iter().any(|user| user.username == newname) {
                            return Err(Error::Forbidden(Some(String::from("Username is already in use!"))))
                        }

                        user.username = newname.clone();
                        
                        let user_insert_vec = match serde_json::to_vec(&user) {
                            Ok(result) => result,
                            Err(_) => return Err(Error::InternalError(None))
                        };

                        match user_database.insert(newname, user_insert_vec) {
                            Ok(_) => {
                                if let Err(_) = user_database.flush() {
                                    return Err(Error::InternalError(Some(String::from("Failed to update backend database"))))
                                };
                                Ok(Status::Ok)
                            },
                            Err(_) => Err(Error::InternalError(None))
                        }
                    },
                    Err(_) => Err(Error::InternalError(None)) 
                }
            },
            Err(_) => Err(Error::Forbidden(None))
        };
    }

    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account's password"),
            (status = 403, description = "An authentication issue has occured when attemping to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/update/password?<username>&<password>&<new_password>")]
    pub async fn update_password(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        new_password: String
    ) -> Result<Status, Error> {
        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let user_vec = match user_database.get(&username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Couldn't find user associated with username"))))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec.to_vec())) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                match user_database.fetch_and_update(user.username, |option_vec| {
                    let user_vec = option_vec.unwrap();
                    let mut user: User = serde_json::from_str(&String::from_utf8_lossy(&user_vec.to_vec())).unwrap();
                    
                    let salt = SaltString::generate(&mut OsRng);
                    user.password = Pbkdf2.hash_password(new_password.as_bytes(), &salt).unwrap().to_string();

                    Some(IVec::from(serde_json::to_vec(&user).unwrap()))
                }) {
                    Ok(_) => {
                        if let Err(_) = user_database.flush() {
                            return Err(Error::InternalError(Some(String::from("Failed to update backend database"))))
                        };
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(Error::InternalError(None))
                }
            },
            Err(_) => Err(Error::Forbidden(None))
        };
    }
}
