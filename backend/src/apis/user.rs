#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::{Config, database::{database::{User, Invite, Media}}, Error};
    
    use rocket::{
        http::Status,
        State,
        get, delete, post, put,
        serde::json::Json, response::status
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
        /// The invited user's username
        invitee_username: Option<String>,
        /// When the invite was used
        #[schema(value_type = String)]
        invitee_date: Option<DateTime::<Utc>>,
        /// Invite creator's username
        creator_username: String,
        /// When invite was created
        #[schema(value_type = String)]
        creation_date: DateTime::<Utc>,
        /// Whether or not the invitation has been used
        used: bool
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserInvite {
        /// Uniquely generated user invite key
        invite: String
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserApiKey {
        /// User media api key
        key: String
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserList {
        /// List of users
        users: Vec<String>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserInfo {
        /// User's account name
        username: String,
        /// User's account creation date
        #[schema(value_type = String)]
        creation_date: DateTime::<Utc>,
        /// Array of upload ids uploaded by the user
        uploads: Vec<String>,
        /// Whether the user is an admin or not 
        admin: bool,
        /// Invite key used to invite the user
        invite_key: Option<String>
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct UserCredentials {
        /// User's account name
        username: String,
        /// User's account password
        password: String
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct UserRegistration {
        /// User's credentials
        user_credentials: UserCredentials,
        /// User's optional invite
        invite: Option<String>
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct UserUpdateUsername {
        /// User's credentials
        user_credentials: UserCredentials,
        /// User's new updated name
        newname: String
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct UserUpdatePassword {
        /// User's old credentials
        user_credentials: UserCredentials,
        /// User's new updated password
        new_password: String,
        /// Whether or not to generate a new api key
        new_api_key: Option<bool>
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct InviteInfoRequest {
        /// Invite key
        invite_key: String
    }

    /// Create's a user account
    #[utoipa::path(
        post,
        context_path = "/api/user",
        request_body = UserRegistration,
        responses(
            (status = 200, description = "Successfully registered account"),
            (status = 401, description = "An unauthorized request has been attempted", body = Error),
            (status = 403, description = "An internal issue has occurred when attempting to register an account", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/register", data = "<registration>")]
    pub async fn register(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        registration: Json<UserRegistration>
    ) -> Result<Status, status::Custom<Json<Error>>> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };
        
        if !config.allow_user_registration {
            return Err(status::Custom(Status::MethodNotAllowed, Json(Error {
                error: String::from("User registration is disabled on this server!")
            })))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => {
                result
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
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
            let optional_invite = match registration.invite.clone() {
                Some(key) => {
                    match invite_database.contains_key(&key) {
                        Ok(has_key) => {
                            if has_key {
                                match invite_database.get(key) {
                                    Ok(result) => { 
                                        result 
                                    },
                                    Err(_) => {
                                        return Err(status::Custom(Status::Unauthorized, Json(Error {
                                            error: String::from("Invitation key does not exist in the database!")
                                        })))
                                    }   
                                }
                            } else {
                                return Err(status::Custom(Status::Unauthorized, Json(Error {
                                    error: String::from("Invitation key does not exist in the database!")
                                })))
                            }
                        },
                        Err(_) => return Err(status::Custom(Status::Unauthorized, Json(Error {
                            error: String::from("An error occurred while looking for your invitation key!")
                        })))
                    }
                },
                None => return Err(status::Custom(Status::Unauthorized, Json(Error {
                    error: String::from("An invitation key is required to register!")
                })))
            };

            option_invite = match optional_invite {
                Some(invite_vec) => {
                    match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
                        Ok(result) => Some(result),
                        Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                            error: String::from("An internal error on the server's end has occurred")
                        })))
                    }
                },
                None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                    error: String::from("An internal error on the server's end has occurred")
                })))
            };
        }
        
        if let Some(invite) = &option_invite { 
            if invite.used {
                return Err(status::Custom(Status::Forbidden, Json(Error {
                    error: String::from("Invitation has already been used!")
                })))
            }
        }
        
        if users.iter().any(|user| user.username == registration.user_credentials.username) {
            return Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("Username is already in use!")
            })))
        }
        
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = match Pbkdf2.hash_password(registration.user_credentials.password.as_bytes(), &salt) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        }.to_string();
        
        let user = User {
            username: registration.user_credentials.username.clone(),
            creation_date: chrono::offset::Utc::now(),
            password: password_hash,
            uploads: Vec::new(),
            api_key: Alphanumeric.sample_string(&mut OsRng, 48),
            admin: users.is_empty() && config.first_user_admin,
            invite_key: {
                if config.use_invite_keys && !users.is_empty() {
                    registration.invite.clone()
                } else {
                    None
                }
            }
        };
        
        println!("{:#?}", user);

        let user_vec = match serde_json::to_vec(&user) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };
        
        match user_database.insert(user.username, user_vec) { 
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        match user_database.flush() {
            Ok(result) => {
                if let Some(mut invite) = option_invite {
                    invite = Invite {
                        invitee_username: Some(registration.user_credentials.username.clone()),
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
                                return Err(status::Custom(Status::InternalServerError, Json(Error {
                                    error: String::from("Failed to update backend database")
                                })))
                            };
                        },
                        Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                            error: String::from("An internal error on the server's end has occurred")
                        })))
                    }

                }
                result
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to add user to database")
            })))
        };
        Ok(Status::Ok)
    }

    /// Grabs media api-key for a user account
    #[utoipa::path(
        post,
        context_path = "/api/user",
        request_body = UserCredentials,
        responses(
            (status = 200, description = "Successfully logged in account", body = UserApiKey),
            (status = 403, description = "An internal issue has occurred when attempting to register an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/login", data = "<credentials>")]
    pub async fn login(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        credentials: Json<UserCredentials>
    ) -> Result<Json<UserApiKey>, status::Custom<Json<Error>>> {
        let user_vec = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(user_tree) => {
                        match user_tree.get(&credentials.username) {
                            Ok(vec) => {
                                match vec {
                                    Some(result) => result,
                                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                                        error: String::from("An internal error on the server's end has occurred")
                                    })))
                                }
                            },
                            
                            Err(_) => return Err(status::Custom(Status::Forbidden, Json(Error {
                                error: String::from("Password or username is not correct")
                            })))
                        }
                    },
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        return match Pbkdf2.verify_password(credentials.password.as_bytes(), &password_hash) {
            Ok(_) => {
                let user_key = UserApiKey {
                    key: user.api_key
                };

                Ok(Json(user_key))
            },
            Err(_) => Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("Password or username is not correct")
            })))
        };
    }

    /// Permanently deletes user account
    /// along with all media on the instance
    #[utoipa::path(
        delete,
        context_path = "/api/user",
        request_body = UserCredentials,
        responses(
            (status = 200, description = "Successfully deleted account"),
            (status = 403, description = "An authentication issue has occurred when attempting to delete an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[delete("/delete", data = "<credentials>")]
    pub async fn delete(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        credentials: Json<UserCredentials>
    ) -> Result<Status, status::Custom<Json<Error>>> {
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

        let user_vec = match user_database.get(&credentials.username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Couldn't find user associated with username")
            })))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        return match Pbkdf2.verify_password(credentials.password.as_bytes(), &password_hash) {
            Ok(_) => {
                let media_database = match database.open_tree("media") {
                    Ok(result) => result,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                };

                let medias: Vec<String> = media_database
                    .iter()
                    .filter_map(|item| item.ok())
                    .filter_map(|item| {
                        let result: Media = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                        Ok(result) => result,
                            Err(_) => return None
                        };
                        Some(result)
                    })
                    .filter(|media| media.author_username == credentials.username)
                    .map(|media| media.id)
                    .collect::<Vec<_>>();
                
                for media_id in medias {
                    if media_database.remove(media_id).is_err() {
                        continue;
                    }
                }

                match user_database.remove(credentials.username.clone()) {
                    Ok(_) => {
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("Failed to remove user account from database")
                    })))
                }
            },
            Err(_) => Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("Invalid or incorrect credentials provided")
            })))
        };
    }

    /// Change or update a user's associated username
    #[utoipa::path(
        put,
        context_path = "/api/user",
        request_body = UserUpdateUsername,
        responses(
            (status = 200, description = "Successfully updated account username"),
            (status = 400, description = "The Client has sent a badly formed request", body = Error),
            (status = 403, description = "An authentication issue has occurred when attempting to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[put("/update/username", data = "<body>")]
    pub async fn update_username(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        body: Json<UserUpdateUsername>
    ) -> Result<Status, status::Custom<Json<Error>>> {
        if body.user_credentials.username == body.newname {
            return Err(status::Custom(Status::BadRequest, Json(Error {
                error: String::from("Invalid request, cannot change name to original name")
            })))
        }

        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let user_vec = match user_database.get(&body.user_credentials.username) {
            Ok(result) => {
                match result {
                    Some(result) => {
                        result
                    },
                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Couldn't find user associated with username")
            })))
        };

        let mut user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        return match Pbkdf2.verify_password(body.user_credentials.password.as_bytes(), &password_hash) {
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

                        if users.iter().any(|user| user.username == body.newname) {
                            return Err(status::Custom(Status::Forbidden, Json(Error {
                                error: String::from("Username is already in use!")
                            })))
                        }

                        user.username = body.newname.clone();
                        
                        let user_insert_vec = match serde_json::to_vec(&user) {
                            Ok(result) => result,
                            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                                error: String::from("An internal error on the server's end has occurred")
                            })))
                        };

                        match user_database.insert(body.newname.clone(), user_insert_vec) {
                            Ok(_) => {
                                if user_database.flush().is_err() {
                                    return Err(status::Custom(Status::InternalServerError, Json(Error {
                                        error: String::from("Failed to update backend database")
                                    })))
                                }

                                Ok(Status::Ok)
                            },
                            Err(_) => Err(status::Custom(Status::InternalServerError, Json(Error {
                                error: String::from("An internal error on the server's end has occurred")
                            })))
                        }
                    },
                    Err(_) => Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("An authentication issue has occurred on the client's end")
            })))
        };
    }

    /// Change or update a user's associated password
    #[utoipa::path(
        put,
        context_path = "/api/user",
        request_body = UserUpdatePassword,
        responses(
            (status = 200, description = "Successfully updated account's password"),
            (status = 403, description = "An authentication issue has occurred when attempting to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[put("/update/password", data = "<body>")]
    pub async fn update_password(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        body: Json<UserUpdatePassword>
    ) -> Result<Status, status::Custom<Json<Error>>> {
        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let user_vec = match user_database.get(&body.user_credentials.username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Couldn't find user associated with username")
            })))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        return match Pbkdf2.verify_password(body.user_credentials.password.as_bytes(), &password_hash) {
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

                    user.password = match Pbkdf2.hash_password(body.new_password.as_bytes(), &salt) {
                        Ok(result) => result.to_string(),
                        Err(_) => return None
                    };

                    if let Some(new_key) = body.new_api_key {
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
                            return Err(status::Custom(Status::InternalServerError, Json(Error {
                                error: String::from("Failed to update backend database")
                            })))
                        };
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("An authentication issue has occurred on the client's end")
            })))
        };
    }

    /// Generates or create's a user invitation
    /// to be used when registering for a account
    #[utoipa::path(
        post,
        context_path = "/api/user",
        request_body = UserCredentials,
        responses(
            (status = 200, description = "Successfully created an invite", body = UserInvite),
            (status = 403, description = "A issue has occurred when attempting to create an invite", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/generate/invite", data = "<credentials>")]
    pub async fn generate_invite(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        credentials: Json<UserCredentials>
    ) -> Result<Json<UserInvite>, status::Custom<Json<Error>>> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };
        
        if !config.use_invite_keys {
            return Err(status::Custom(Status::MethodNotAllowed, Json(Error {
                error: String::from("Invitations are disabled on this server!")
            })))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let user_vec = match user_database.get(&credentials.username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Couldn't find user associated with username")
            })))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        return match Pbkdf2.verify_password(credentials.password.as_bytes(), &password_hash) {
            Ok(_) => {
                let invite = Invite {
                    key: Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
                    invitee_username: None,
                    invitee_date: None,
                    creation_date: chrono::offset::Utc::now(),
                    creator_username: credentials.username.clone(),
                    used: false
                };

                println!("{:#?}", invite);

                let invite_vec = match serde_json::to_vec(&invite) {
                    Ok(result) => result,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                };
                
                match invite_database.insert(&invite.key, invite_vec) { 
                    Ok(result) => result,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                };

                match user_database.flush() {
                    Ok(_) => {
                        let user_invite = UserInvite {
                            invite: invite.key
                        };

                        Ok(Json(user_invite))
                    }
                    Err(_) => Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            }
            Err(_) => Err(status::Custom(Status::Forbidden, Json(Error {
                error: String::from("An authentication issue has occurred on the client's end")
            })))
        }
    }


    /// Grabs information about an invite
    /// such as author, creation date, used status, and invitee name/date
    #[utoipa::path(
        post,
        context_path = "/api/user",
        request_body = InviteInfoRequest,
        responses(
            (status = 200, description = "Successfully grabbed invite information", body = InviteInfo),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/info/invite", data = "<body>")]
    pub async fn invite_info(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        body: Json<InviteInfoRequest>
    ) -> Result<Json<InviteInfo>, status::Custom<Json<Error>>> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Failed to access backend database")
            })))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        let invite_vec = match invite_database.get(&body.invite_key) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(status::Custom(Status::InternalServerError, Json(Error {
                        error: String::from("An internal error on the server's end has occurred")
                    })))
                }
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("Couldn't find user associated with username")
            })))
        };

        let invite: Invite = match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
            Ok(result) => result,
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(Error {
                error: String::from("An internal error on the server's end has occurred")
            })))
        };

        Ok(Json(InviteInfo {
            invitee_username: invite.invitee_username,
            invitee_date: invite.invitee_date,
            creator_username: invite.creator_username,
            creation_date: invite.creation_date,
            used: invite.used
        }))
    }

    /// Lists all users on this instance
    #[utoipa::path(
        get,
        context_path = "/api/user",
        responses(
            (status = 200, description = "Successfully grabbed all users", body = UserList),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/list")]
    pub async fn list(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>
    ) -> Result<Json<UserList>, status::Custom<Json<Error>>> {
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

        let usernames = user_database.iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                Ok(result) => result,
                Err(_) => None
            })
            .map(|user: User| user.username)
            .collect::<Vec<String>>();

        let user_list = UserList {
            users: usernames
        };

        Ok(Json(user_list))
    }

    /// Grabs information from a user's account
    /// based on media key's
    #[utoipa::path(
        post,
        context_path = "/api/user",
        request_body = UserApiKey,
        responses(
            (status = 200, description = "Successfully grabbed user info", body = UserInfo),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/info", data = "<body>")]
    pub async fn info(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        body: Json<UserApiKey>
    ) -> Result<Json<UserInfo>, status::Custom<Json<Error>>> {
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

        let user = user_database.iter()
            .filter_map(|item| item.ok())
            .map(|item| {
                let result: User = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                    Ok(result) => result,
                    Err(_) => return None
                };
                Some(result)
            }).find_map(|user| {
                match user {
                    Some(result) => {
                        if result.api_key == body.key {
                            return Some(result)
                        }
                        None
                    },
                    None => None
                }
            });

        return match user {
            Some(user) => {
                let user_info = UserInfo {
                    username: user.username,
                    creation_date: user.creation_date,
                    uploads: user.uploads,
                    admin: user.admin,
                    invite_key: user.invite_key
                };

                Ok(Json(user_info))
            }
            None => Err(status::Custom(Status::Unauthorized, Json(Error {
                error: String::from("Api key is not valid and or does not exist!")
            })))
        };
    }
}