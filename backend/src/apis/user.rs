#[allow(non_snake_case)]
pub mod User {
    use std::sync::{Arc, Mutex};

    use crate::{Config, database::{database::{User, Invite, Media}}, Error};
    
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
        #[schema(example = "Whether or not the invitation has been used")]
        used: bool
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserInvite {
        #[schema(example = "Uniquely generated user invite key")]
        invite: String
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserKey {
        #[schema(example = "User media key")]
        key: String
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserList {
        #[schema(example = "List of users")]
        users: Vec<String>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct UserInfo {
        username: String,
        creation_date: DateTime::<Utc>,
        uploads: Vec<String>,
        admin: bool,
        invite_key: Option<String>
    }

    /// Create's a user account
    #[utoipa::path(
        post,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully registered account"),
            (status = 401, description = "An unauthorized request has been attempted", body = Error),
            (status = 403, description = "An internal issue has occurred when attempting to register an account", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
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
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };
        
        if !config.allow_user_registration {
            return Err(Error::NotAllowed(String::from("User registration is disabled on this server!")))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => {
                result
            },
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                        Err(_) => return Err(Error::Unauthorized(String::from("An error occurred while looking for your invitation key!"))) 
                    }
                },
                None => return Err(Error::Unauthorized(String::from("An invitation key is required to register!")))
            };

            option_invite = match optional_invite {
                Some(invite_vec) => {
                    match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
                        Ok(result) => Some(result),
                        Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                    }
                },
                None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
            };
        }
        
        if let Some(invite) = &option_invite { 
            if invite.used {
                return Err(Error::Forbidden(String::from("Invitation has already been used!")))
            }
        }
        
        if users.iter().any(|user| user.username == username) {
            return Err(Error::Forbidden(String::from("Username is already in use!")))
        }
        
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = match Pbkdf2.hash_password(password.as_bytes(), &salt) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };
        
        match user_database.insert(user.username, user_vec) { 
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                                return Err(Error::InternalError(String::from("Failed to update backend database")))
                            };
                        },
                        Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                    }

                }
                result
            },
            Err(_) => return Err(Error::InternalError(String::from("Failed to add user to database")))
        };
        Ok(Status::Ok)
    }

    /// Grabs media api-key for a user account
    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully logged in account"),
            (status = 403, description = "An internal issue has occurred when attempting to register an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/login?<username>&<password>")]
    pub async fn login(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<Json<UserKey>, Error> {
        let user_vec = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(user_tree) => {
                        match user_tree.get(&username) {
                            Ok(vec) => {
                                match vec {
                                    Some(result) => result,
                                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                                }
                            },
                            Err(_) => return Err(Error::Forbidden(String::from("Password or username is not correct")))
                        }
                    },
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                let user_key = UserKey {
                    key: user.api_key
                };

                Ok(Json(user_key))
            },
            Err(_) => Err(Error::Forbidden(String::from("Password or username is not correct")))
        };
    }

    /// Permanently deletes user account
    /// along with all media on the instance
    #[utoipa::path(
        delete,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully deleted account"),
            (status = 403, description = "An authentication issue has occurred when attempting to delete an account", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[delete("/delete?<username>&<password>")]
    pub async fn delete(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<Status, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user_vec = match user_database.get(&username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find user associated with username")))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        return match Pbkdf2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => {
                let media_database = match database.open_tree("media") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                    .filter(|media| media.author_username == username)
                    .map(|media| media.id)
                    .collect::<Vec<_>>();
                
                for media_id in medias {
                    if media_database.remove(media_id).is_err() {
                        continue;
                    }
                }

                match user_database.remove(username) {
                    Ok(_) => {
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(Error::InternalError(String::from("Failed to remove user account from database")))
                }
            },
            Err(_) => Err(Error::Forbidden(String::from("Invalid or incorrect credentials provided")))
        };
    }

    /// Change or update a user's associated username
    #[utoipa::path(
        put,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account username"),
            (status = 400, description = "The Client has sent a badly formed request", body = Error),
            (status = 403, description = "An authentication issue has occurred when attempting to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
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
            return Err(Error::BadRequest(String::from("Invalid request, cannot change name to original name")))
        }

        let user_database = match database_store.lock() {
            Ok(database) => {
                match database.open_tree("user") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user_vec = match user_database.get(username) {
            Ok(result) => {
                match result {
                    Some(result) => {
                        result
                    },
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find user associated with username")))
        };

        let mut user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                            return Err(Error::Forbidden(String::from("Username is already in use!")))
                        }

                        user.username = newname.clone();
                        
                        let user_insert_vec = match serde_json::to_vec(&user) {
                            Ok(result) => result,
                            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        };

                        match user_database.insert(newname, user_insert_vec) {
                            Ok(_) => {
                                if user_database.flush().is_err() {
                                    return Err(Error::InternalError(String::from("Failed to update backend database")))
                                }

                                Ok(Status::Ok)
                            },
                            Err(_) => Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        }
                    },
                    Err(_) => Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => Err(Error::Forbidden(String::from("An authentication issue has occurred on the client's end")))
        };
    }

    /// Change or update a user's associated password
    #[utoipa::path(
        put,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully updated account's password"),
            (status = 403, description = "An authentication issue has occurred when attempting to update account username", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
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
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user_vec = match user_database.get(&username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find user associated with username")))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                            return Err(Error::InternalError(String::from("Failed to update backend database")))
                        };
                        Ok(Status::Ok)
                    },
                    Err(_) => Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => Err(Error::Forbidden(String::from("An authentication issue has occurred on the client's end")))
        };
    }

    /// Generates or create's a user invitation
    /// to be used when registering for a account
    #[utoipa::path(
        post,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully created an invite"),
            (status = 403, description = "A issue has occurred when attempting to create an invite", body = Error),
            (status = 405, description = "The api endpoint is not allowed to execute", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/generate/invite?<username>&<password>")]
    pub async fn generate_invite(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: String,
        password: String
    ) -> Result<Json<UserInvite>, Error> {
        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };
        
        if !config.use_invite_keys {
            return Err(Error::NotAllowed(String::from("Invitations are disabled on this server!")))
        }

        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user_vec = match user_database.get(&username) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find user associated with username")))
        };

        let user: User = match serde_json::from_str(&String::from_utf8_lossy(&user_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let password_hash = match PasswordHash::new(&user.password) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };
                
                match invite_database.insert(&invite.key, invite_vec) { 
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                match user_database.flush() {
                    Ok(_) => {
                        let user_invite = UserInvite {
                            invite: invite.key
                        };

                        Ok(Json(user_invite))
                    }
                    Err(_) => Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            }
            Err(_) => Err(Error::Forbidden(String::from("An authentication issue has occurred on the client's end")))
        }
    }


    /// Grabs information about an invite
    /// such as author, creation date, used status, and invitee name/date
    #[utoipa::path(
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully grabbed invite information"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
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
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let invite_database = match database.open_tree("invite") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let invite_vec = match invite_database.get(&invite_key) {
            Ok(result) => {
                match result {
                    Some(result) => result,
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find user associated with username")))
        };

        let invite: Invite = match serde_json::from_str(&String::from_utf8_lossy(&invite_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully grabbed all users"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/list")]
    pub async fn list(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>
    ) -> Result<Json<UserList>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
        get,
        context_path = "/user",
        responses(
            (status = 200, description = "Successfully grabbed user info"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/info?<api_key>")]
    pub async fn info(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        api_key: String
    ) -> Result<Json<UserInfo>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
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
                        if result.api_key == api_key {
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
            None => Err(Error::Unauthorized(String::from("Api key not valid and or does not exist!")))
        };
    }
}