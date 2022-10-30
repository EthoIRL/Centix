#[allow(non_snake_case)]
pub mod Media {
    use std::{sync::{Arc, Mutex}, path::Path, fs::{self, File}, io::Write};

    use crate::{Config::*};
    use crate::Error;
    use crate::database::database::{User, Media as DBMedia};

    use flate2::{write::ZlibEncoder, Compression};
    use rocket::{
        fs::{NamedFile},
        get,
        http::Status,
        serde::json::Json,
        FromForm, State,
        FromFormField, post,
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    use rand::distributions::{Alphanumeric, DistString};
    use rand_core::OsRng;

    use base64::decode;

    use infer::{MatcherType};

    use sled::IVec;

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

    #[derive(FromFormField, ToSchema)]
    pub enum ContentType {
        Video,
        Image,
        Other
    }

    #[utoipa::path(
        get,
        responses(
            (status = 200, description = "Successfully found media")
        ),
        params(
            ("id", example = "HilrvkpJ")
        )
    )]
    #[get("/<id>")]
    pub async fn grab(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        id: String,
    ) -> Option<NamedFile> {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully grabbed all media")
        )
    )]
    #[get("/all?<username>&<content_type>")]
    pub async fn all(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        username: Option<String>,
        content_type: Option<ContentType>
    ) -> Json<Vec<Media>> {
        todo!()
    }

    #[utoipa::path(
        post,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully uploaded media"),
            (status = 400, description = "Server recieved malformed client request", body = Error),
            (status = 403, description = "An authentication issue has occured", body = Error),
            (status = 500, description = "An internal error on the server's end has occured", body = Error)
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
            Err(_) => return Err(Error::InternalError(Some(String::from("Failed to access backend database"))))
        };

        let user_database = match database.open_tree("user") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(None))
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
                        println!("Username: {}, Key: {}", &result.username, &result.api_key);

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
                    Err(_) => return Err(Error::InternalError(None))
                };

                if upload.name.len() as i32 > config.content_name_length {
                    return Err(Error::BadRequest(Some(format!("Name length too long. Maximum of {} characters", config.content_name_length))))
                }

                let upload_data = match decode(body_data.0) {
                    Ok(result) => {
                        result
                    },
                    Err(_) => return Err(Error::InternalError(None))
                };

                if config.content_max_size > 0 {
                    let mb_size = upload_data.len() as i32 / 1000000;
                    if mb_size > config.content_max_size {
                        return Err(Error::BadRequest(Some(format!("File size too big! Maximum of {} megabytes", config.content_max_size))))
                    }
                }
                
                // DONE # TODO: Magic mime type grabber lib & determine extension
                // DONE # TODO: Loseless cold compress the file
                // DONE # TODO: Store file to disk with path based on config.content_directory
                // DONE # TODO: Update the user's uploads to contain id of new media

                let data_type = match infer::get(&upload_data) {
                    Some(result) => {
                        result
                    }
                    None => return Err(Error::InternalError(None)) 
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

                let mut content_directory = match data_type.matcher_type() {
                    MatcherType::VIDEO => {
                        content_path.join("Video") 
                    }
                    MatcherType::IMAGE => {
                        content_path.join("Image")
                    }
                    _ => {
                        content_path.join("Other") 
                    }
                };

                if !content_directory.exists() && fs::create_dir_all(&content_directory).is_err() {
                    return Err(Error::InternalError(None))
                }

                content_directory = content_directory.join(Alphanumeric.sample_string(&mut OsRng, 24));
                let mut content_file = match File::create(&content_directory) {
                    Ok(result) => {
                        result
                    }
                    Err(_) => return Err(Error::InternalError(None))
                };

                if content_file.write_all(&data.0).is_err() {
                    return Err(Error::InternalError(None))
                }

                let media = DBMedia {
                    id: Alphanumeric.sample_string(&mut OsRng, config.content_id_length as usize),
                    name: upload.name,
                    extension: data_type.extension().to_string(),
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
                    Err(_) => return Err(Error::InternalError(None))
                };

                let media_vec = match serde_json::to_vec(&media) {
                    Ok(result) => result,
                    Err(_) => return Err(Error::InternalError(None))
                };

                if media_database.insert(&media.id, media_vec).is_err() {
                    return Err(Error::InternalError(None))
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
                            return Err(Error::InternalError(None))
                        }

                        if user_database.update_and_fetch(&user.username, |_| {
                            Some(IVec::from(match serde_json::to_vec(&user) {
                                Ok(result) => result,
                                Err(_) => return None
                            }))
                        }).is_err() {
                            return Err(Error::InternalError(None))
                        }

                        println!("User: {:#?}", user);

                        if user_database.flush().is_err() {
                            return Err(Error::InternalError(None))
                        }

                        Ok(Status::Ok)
                    },
                    Err(_) => return Err(Error::InternalError(None))
                }
            }
            None => {
                Err(Error::Forbidden(None))
            }
        };
    }

    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully deleted media")
        )
    )]
    #[get("/delete?<id>")]
    pub async fn delete(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        id: String,
    ) -> Status {
        todo!()
    }
}
