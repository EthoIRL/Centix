#[allow(non_snake_case)]
pub mod Media {
    use std::{sync::{Arc, Mutex}, path::Path, fs::{self, File}, io::{Write, Read}};

    use crate::{Config::*};
    use crate::Error;
    use crate::database::database::{User, Media as DBMedia};

    use flate2::{write::{ZlibEncoder, ZlibDecoder}, Compression};
    use rocket::{
        get, 
        http::{Status, Header},
        serde::json::Json,
        FromForm, State,
        FromFormField, post,
        response::Responder, delete
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    use rand::distributions::{Alphanumeric, DistString};
    use rand_core::OsRng;

    use base64::decode;

    use infer::{MatcherType};

    use sled::IVec;

    use chrono::{DateTime, Utc};

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct Media {
        #[schema(example = "HilrvkpJ")]
        id: String
    }

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct UploadParam {
        #[schema(example = "Funny Cat")]
        name: String,
        #[schema(example = "Private video not listed on /all/ endpoint")]
        private: Option<bool>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct ContentInfo {
        #[schema(example = "Etho")]
        author_username: String,
        #[schema(example = "582000 bytes")]
        content_size: i32,
        #[schema(example = "UTC Format")]
        upload_date: DateTime::<Utc>,
        #[schema(example = "Privately listed video")]
        private: bool
    }

    #[derive(Serialize, Deserialize, FromFormField, ToSchema, PartialEq, Eq, Clone, Debug)]
    pub enum ContentType {
        Video,
        Image,
        Other
    }

    #[derive(Responder)]
    pub struct FileResponder<T> {
        inner: T,
        content_disposition: Header<'static>,
    }
    impl<'r, 'o: 'r, T: Responder<'r, 'o>> FileResponder<T> {
        pub fn new(inner: T, file_disposition: String) -> Self {
            FileResponder {
                inner,
                content_disposition: Header::new("content-disposition", file_disposition),
            }
        }
    }

    /// Returns useful media information  
    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully grabbed media information"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        ),
        params(
            ("id", example = "HilrvkpJ")
        )
    )]
    #[get("/info/<id>")]
    pub async fn info(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        id: String,
    ) -> Result<Json<ContentInfo>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let media_vec: IVec = match media_database.get(id) {
            Ok(result) => {
                match result {
                    Some(result) => {
                        result
                    },
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            },
            Err(_) => return Err(Error::InternalError(String::from("Couldn't find media associated with id")))
        };

        let media: DBMedia = match serde_json::from_str(&String::from_utf8_lossy(&media_vec)) {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        Ok(Json(ContentInfo {
            author_username: media.author_username,
            content_size: media.data_size,
            upload_date: media.upload_date,
            private: media.private
        }))
    }

    /// Returns file-disposition based file download
    #[utoipa::path(
        get,
        responses(
            (status = 200, description = "Successfully found media"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        ),
        params(
            ("id", example = "HilrvkpJ")
        )
    )]
    #[get("/<id>")]
    pub async fn grab(
        _config: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        id: String,
    ) -> Result<FileResponder<Vec<u8>>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let media: Option<DBMedia> = media_database.iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| {
                let result: DBMedia = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                    Ok(result) => result,
                    Err(_) => return None
                };
                Some(result)
            })
            .find(|media| media.id == id);

        if let Some(media) = media {
            let mut file = match File::open(media.data_path) {
                Ok(result) => result,
                Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
            };

            let mut upload_data = Vec::new();
            if file.read_to_end(&mut upload_data).is_err() {
                return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
            };

            let data: Vec<u8> = if media.data_compressed {
                let mut writer = Vec::new();
                let mut zlibdecoder = ZlibDecoder::new(writer);
                if zlibdecoder.write_all(&upload_data).is_err() {
                    return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };
                writer = match zlibdecoder.finish() {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };
                writer.to_vec()
            } else {
                upload_data
            };

            let filename_extension = format!("{}.{}", media.name, media.extension);
            Ok(FileResponder::new(data, format!(r#"attachment; filename={};"#, filename_extension)))
        } else {
            Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        }
    }

    /// Grabs all media id's in the form of a list
    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully grabbed all media"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/all?<username>&<content_type>&<api_key>")]
    pub async fn all(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: Option<String>,
        content_type: Option<ContentType>,
        api_key: Option<String>
    ) -> Result<Json<Vec<String>>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let user: Option<User> = if api_key.is_some() {
            user_database
                .iter()
                .filter_map(|item| item.ok())
                .map(|item| {
                    let result: User = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                        Ok(result) => result,
                        Err(_) => return None
                    };
                    Some(result)
                })
                .find_map(|user| {
                    match user {
                        Some(result) => {
                            match &api_key {
                                Some(key) => {
                                    if &result.api_key == key {
                                        return Some(result)
                                    }
                                },
                                None => return None
                            }
                            None
                        },
                        None => None
                    }
                })
        } else {
            None
        };

        let medias: Vec<String> = media_database
            .iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| {
                let result: DBMedia = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                Ok(result) => result,
                    Err(_) => return None
                };
                Some(result)
            })
            .filter(|media| {
                if username.is_none() {
                    return true;
                }

                if let Some(username) = &username {
                    return &media.author_username == username
                }
                false
            })
            .filter(|media| {
                if content_type.is_none() {
                    return true;
                }

                if let Some(content) = &content_type {
                    if content == &media.data_type {
                        return true
                    }
                }
                false
            })
            .filter(|media| {
                if user.is_some() {
                    return true;
                }

                !media.private
            })
            .map(|media| media.id)
            .collect();
        Ok(Json(medias))
    }

    /// Uploads media to a user's account
    /// 
    /// Media data should be in the form of base64 string inside the body 
    #[utoipa::path(
        post,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully uploaded media"),
            (status = 400, description = "Server received malformed client request", body = Error),
            (status = 401, description = "An authentication issue has occurred", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        ),
        params(
            UploadParam
        ),
        request_body = Json<String>
    )]
    #[post("/upload?<api_key>&<upload..>", data = "<body_data>")]
    pub async fn upload(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        upload: UploadParam,
        body_data: Json<String>,
        api_key: String
    ) -> Result<Status, Error> {
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
            Some(mut user) => {
                let config = match config_store.lock() {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                if upload.name.len() as i32 > config.content_name_length {
                    return Err(Error::BadRequest(format!("Name length too long. Maximum of {} characters", config.content_name_length)))
                }

                let upload_data = match decode(body_data.0) {
                    Ok(result) => {
                        result
                    },
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                if config.content_max_size > 0 {
                    let mb_size = upload_data.len() as i32 / 1000000;
                    if mb_size > config.content_max_size {
                        return Err(Error::BadRequest(format!("File size too big! Maximum of {} megabytes", config.content_max_size)))
                    }
                }

                let data_type = match infer::get(&upload_data) {
                    Some(result) => {
                        result
                    }
                    None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred"))) 
                };

                let data: (Vec<u8>, bool) = if config.store_compressed {
                    let zlib_encoder = ZlibEncoder::new(upload_data.clone(), Compression::best());
                    match zlib_encoder.finish() {
                        Ok(result) => { 
                            if result.len() > upload_data.len() {
                                (upload_data, false)
                            } else {
                                (result, true)
                            } 
                        },
                        Err(_) => (upload_data, false)
                    }
                } else {
                    (upload_data, false)
                };

                let content_path = match &config.content_directory {
                    Some(result) => {
                        Path::new(result)
                    },
                    None => { 
                        Path::new(".\\")
                    }
                }.join("content");

                let mut content_type = ContentType::Other;
                let mut content_directory = match data_type.matcher_type() {
                    MatcherType::Video => {
                        content_type = ContentType::Video;
                        content_path.join("Video") 
                    }
                    MatcherType::Image => {
                        content_type = ContentType::Image;
                        content_path.join("Image")
                    }
                    _ => {
                        content_path.join("Other") 
                    }
                };

                if !content_directory.exists() && fs::create_dir_all(&content_directory).is_err() {
                    return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }

                content_directory = content_directory.join(Alphanumeric.sample_string(&mut OsRng, 24));
                let mut content_file = match File::create(&content_directory) {
                    Ok(result) => {
                        result
                    }
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                if content_file.write_all(&data.0).is_err() {
                    return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }

                let media = DBMedia {
                    id: Alphanumeric.sample_string(&mut OsRng, config.content_id_length as usize),
                    name: upload.name,
                    extension: data_type.extension().to_string(),
                    data_type: content_type,
                    data_path: content_directory,
                    data_size: data.0.len() as i32,
                    upload_date: chrono::offset::Utc::now(),
                    data_compressed: data.1,
                    author_username: user.username.clone(),
                    private: upload.private.unwrap_or(false)
                };

                println!("Media: {:#?}", media);

                let media_database = match database.open_tree("media") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                let media_vec = match serde_json::to_vec(&media) {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                if media_database.insert(&media.id, media_vec).is_err() {
                    return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }

                match media_database.flush() {
                    Ok(_) => {
                        user.uploads.push(media.id);
                        if user_database.update_and_fetch(&user.username, |_| {
                            Some(IVec::from(match serde_json::to_vec(&user) {
                                Ok(result) => result,
                                Err(_) => return None
                            }))
                        }).is_err() {
                            return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        }

                        if user_database.update_and_fetch(&user.username, |_| {
                            Some(IVec::from(match serde_json::to_vec(&user) {
                                Ok(result) => result,
                                Err(_) => return None
                            }))
                        }).is_err() {
                            return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        }

                        println!("User: {:#?}", user);

                        if user_database.flush().is_err() {
                            return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        }

                        Ok(Status::Ok)
                    },
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                }
            }
            None => {
                Err(Error::Unauthorized(String::from("Invalid or wrong credentials provided")))
            }
        };
    }

    /// Permanently deletes media from instance
    #[utoipa::path(
        delete,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully deleted media"),
            (status = 401, description = "Unauthorized deletion", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error),
        )
    )]
    #[delete("/delete?<api_key>&<id>")]
    pub async fn delete(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        id: String,
        api_key: String
    ) -> Result<Status, Error> {
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
            Some(mut user) => {
                let media_database = match database.open_tree("media") {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                let media_vec = match media_database.get(&id) {
                    Ok(result) => {
                        match result {
                            Some(result) => result,
                            None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                        }
                    },
                    Err(_) => return Err(Error::InternalError(String::from("Couldn't find media associated with id")))
                };

                let media: DBMedia = match serde_json::from_str(&String::from_utf8_lossy(&media_vec)) {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                };

                if media.author_username == user.username {
                    match media_database.remove(&id) {
                        Ok(_) => {
                            user.uploads.retain(|upload| upload == &id);
                            match user_database.update_and_fetch(&user.username, |_| {
                                Some(IVec::from(match serde_json::to_vec(&user) {
                                    Ok(result) => result,
                                    Err(_) => return None
                                }))
                            }) {
                                Ok(_) => {
                                    Ok(Status::Ok)
                                },
                                Err(_) => Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                            }
                        },
                        Err(_) => Err(Error::InternalError(String::from("Failed delete media from database")))
                    }
                } else {
                    Err(Error::Unauthorized(String::from("Media does not belong to associated api key!")))
                }
            },
            None => Err(Error::Unauthorized(String::from("Api key is invalid and does not exist")))
        }
    }
}
