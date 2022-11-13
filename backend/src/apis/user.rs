#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::{Config::*, database::{database::{User, Invite}}, Error};
    
    use rocket::{
        http::Status,
        State,
        get, delete, post, put,
        serde::json::Json
    };
    use sled::IVec;

    use pbkdf2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString
        },
        Pbkdf2
    };

    use rand::distributions::{Alphanumeric, DistString};

    use serde::{Serialize, Deserialize};
    use utoipa::{IntoParams, ToSchema};

    use chrono::{DateTime, Utc};

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct InviteInfo {
        #[schema(example = "The invited user's username")]
        invitee_username: Option<String>,
        #[schema(example = "When the invite was used")]
        invitee_date: Option<DateTime::<Utc>>,
        #[schema(example = "Invite creator's username")]
        creator_username: String,
        #[schema(example = "When invite was created")]
        creation_date: DateTime::<Utc>,
        #[schema(example = "Whether or not the inviation has been used")]
        used: bool
    }

    #[utoipa::path(
        post,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully registered account"),
            (status = 401, description = "An unauthurized request has been attempted", body = Error),
            (status = 403, description = "An internal issue has occured when attemping to register an account", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[post("/register?<username>&<password>&<invite>")]
    pub async fn register(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        invite: Option<String>
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

        let users: Vec<User> = user_database.iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                Ok(result) => result,
                Err(_) => None
            })
            .collect::<Vec<_>>();

        let mut option_invite: Option<Invite> = None;

        if config.use_invite_keys && !users.is_empty() {
            let optional_invite = match invite.clone() {
                Some(key) => {
                    match invite_database.contains_key(&key) {
                        Ok(has_key) => {
                            if has_key {
                                match invite_database.get(key) {
                                    Ok(result) => { 
                                        result 
                                    },
                                    Err(_) => {
                                        return Err(Error::Unauthorized(String::from("Invitation key does not exist in the database!"))) 
                                    }   
                                }
                            } else {
                                return Err(Error::Unauthorized(String::from("Invitation key does not exist in the database!")))
                            }
                        },
                        Err(_) => return Err(Error::Unauthorized(String::from("An error occured while looking for your invitiation key!"))) 
                    }
                },
                None => return Err(Error::Unauthorized(String::from("An invitation key is required to register!")))
            };

            option_invite = match optional_invite {
                Some(invite_vec) => {
                    match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
                        Ok(result) => Some(result),
                        Err(_) => return Err(Error::InternalError(None))
                    }
                },
                None => return Err(Error::InternalError(None))
            };
        }
        
        if let Some(invite) = &option_invite { 
            if invite.used {
                return Err(Error::Forbidden(Some(String::from("Invitation has already been used!"))))
            }
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
            api_key: Alphanumeric.sample_string(&mut OsRng, 48),
            admin: users.is_empty() && config.first_user_admin,
            invite_key: {
                if config.use_invite_keys && !users.is_empty() {
                    invite
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
                if let Some(mut invite) = option_invite {
                    invite = Invite {
                        invitee_username: Some(username),
                        invitee_date: Some(chrono::offset::Utc::now()),
                        used: true,
                        ..invite
                    };

                    match invite_database.update_and_fetch(&invite.key, |_| {
                        Some(IVec::from(match serde_json::to_vec(&invite) {
                            Ok(result) => result,
                            Err(_) => return None
                        }))
                    }) {
                        Ok(_) => {
                            if invite_database.flush().is_err() {
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

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                Ok(user.api_key)
            },
            Err(_) => Err(Error::Forbidden(Some(String::from("Password or username is not correct"))))
        };
    }

    #[utoipa::path(
        delete,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully deleted account"),
            (status = 403, description = "An authentication issue has occured when attemping to delete an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[delete("/delete?<username>&<password>")]
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

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                match user_database.remove(username) {
                    Ok(_) => {
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(Error::InternalError(Some(String::from("Failed delete user account"))))
                }
            },
            Err(_) => Err(Error::Forbidden(None))
        };
    }

    #[utoipa::path(
        put,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account username"),
            (status = 400, description = "The Client has sent a badly formed request", body = Error),
            (status = 403, description = "An authentication issue has occured when attemping to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[put("/update/username?<username>&<password>&<newname>")]
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

        let mut user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
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
                        let users: Vec<User> = user_database.iter()
                            .filter_map(|item| item.ok())
                            .filter_map(|item| match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                                Ok(result) => result,
                                Err(_) => None
                            })
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
                                if user_database.flush().is_err() {
                                    return Err(Error::InternalError(Some(String::from("Failed to update backend database"))))
                                }

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
        put,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account's password"),
            (status = 403, description = "An authentication issue has occured when attemping to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[put("/update/password?<username>&<password>&<new_password>&<new_api_key>")]
    pub async fn update_password(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String,
        new_password: String,
        new_api_key: Option<bool>
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

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
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
                    let user_vec = match option_vec {
                        Some(result) => result,
                        None => return None
                    };

                    let mut user: User = match serde_json::from_str(&String::from_utf8_lossy(user_vec)) {
                        Ok(result) => result,
                        Err(_) => return None
                    };
                    
                    let salt = SaltString::generate(&mut OsRng);

                    user.password = match Pbkdf2.hash_password(new_password.as_bytes(), &salt) {
                        Ok(result) => result.to_string(),
                        Err(_) => return None
                    };

                    if let Some(new_key) = new_api_key {
                        if new_key {
                            user.api_key = Alphanumeric.sample_string(&mut OsRng, 48);
                        }
                    }

                    Some(IVec::from(match serde_json::to_vec(&user) {
                        Ok(result) => result,
                        Err(_) => return None
                    }))
                }) {
                    Ok(_) => {
                        if user_database.flush().is_err() {
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

    #[utoipa::path(
        post,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully created an invite"),
            (status = 403, description = "A issue has occured when attemping to create an invite", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[post("/generate/invite?<username>&<password>")]
    pub async fn generate_invite(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<String, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };
        
        if !config.use_invite_keys {
            return Err(Error::NotAllowed(String::from("Invitations are disabled on this server!")))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
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

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                let invite = Invite {
                    key: Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
                    invitee_username: None,
                    invitee_date: None,
                    creation_date: chrono::offset::Utc::now(),
                    creator_username: username,
                    used: false
                };

                println!("{:#?}", invite);

                let invite_vec = match serde_json::to_vec(&invite) {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                };
                
                match invite_database.insert(&invite.key, invite_vec) { 
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                };

                match user_database.flush() {
                    Ok(_) => {
                        Ok(invite.key)
                    }
                    Err(_) => Err(Error::InternalError(None))
                }
            }
            Err(_) => Err(Error::Forbidden(None))
        }
    }



    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully grabbed invite imformation"),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
        )
    )]
    #[get("/info/invite?<invite_key>")]
    pub async fn invite_info(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        invite_key: String
    ) -> Result<Json<InviteInfo>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        let invite_vec = match invite_database.get(&invite_key) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(None))
                }
            },
            Err(_) => return Err(Error::InternalError(Some(String::from("Couldn't find user associated with username"))))
        };

        let invite: Invite = match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
        };

        Ok(Json(InviteInfo {
            invitee_username: invite.invitee_username,
            invitee_date: invite.invitee_date,
            creator_username: invite.creator_username,
            creation_date: invite.creation_date,
            used: invite.used
        }))
    }
}
