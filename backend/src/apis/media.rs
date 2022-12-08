#[allow(non_snake_case)]
pub mod Media {
    use std::{sync::{Arc, Mutex}, path::Path, fs::{self, File}, io::{Write, Read}};

    use crate::{Error, Config};
    use crate::database::database::{User, Media as DBMedia};

    use flate2::{write::{ZlibEncoder, ZlibDecoder}, Compression};
    use itertools::Itertools;
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
        #[schema(example = "Funny cat video")]
        name: String,
        #[schema(example = "Private media not listed on /all/ endpoint")]
        private: Option<bool>,
        #[schema(example = "Tags relating to the media")]
        tags: Option<Vec<String>>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct ContentInfo {
        #[schema(example = "Etho")]
        author_username: String,
        #[schema(example = "Funny cat video")]
        content_name: String,
        #[schema(example = "582000 bytes")]
        content_size: i32,
        #[schema(example = "UTC Format")]
        upload_date: DateTime::<Utc>,
        #[schema(example = "Privately listed media")]
        private: bool,
        #[schema(example = "Tags associated to media")]
        tags: Option<Vec<String>>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct ContentFound {
        #[schema(example = "List of ids found")]
        ids: Vec<String>
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct ContentId {
        #[schema(example = "HilrvkpJ")]
        id: String
    }

    #[derive(Serialize, Deserialize, IntoParams, ToSchema, Clone)]
    pub struct ContentTags {
        #[schema(example = "List of all in use tags")]
        tags: Vec<String>
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
            content_name: media.name,
            content_size: media.data_size,
            upload_date: media.upload_date,
            private: media.private,
            tags: media.tags
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

    /// Finds all media id's in the form of a list
    /// based on optional queries patterns
    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully found all media"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/find?<username>&<content_type>&<api_key>&<tags>")]
    pub async fn find(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        username: Option<String>,
        content_type: Option<ContentType>,
        api_key: Option<String>,
        tags: Option<Vec<String>>
    ) -> Result<Json<ContentFound>, Error> {
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
            .filter(|media| {
                if tags.is_none() {
                    return true
                }

                if let Some(tags) = &tags {
                    if tags.is_empty() {
                        return true;
                    }

                    for find_tag in tags {
                        if media.tags.is_none() {
                            return false;
                        }

                        let tag = find_tag.to_lowercase();
                        
                        if let Some(media_tags) = &media.tags {
                            if media_tags.is_empty() {
                                return false;
                            }

                            if !media_tags.contains(&tag) {
                                return false;
                            }
                        }
                    } 

                    return true;
                }
                false
            })
            .map(|media| media.id)
            .collect();

        let found_content = ContentFound {
            ids: medias
        };
        Ok(Json(found_content))
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
    ) -> Result<Json<ContentId>, Error> {
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

                let mut safe_tags: Option<Vec<String>> = None;

                if let Some(tags) = upload.tags {
                    let sorted_tags: Vec<String> = tags
                    .into_iter()
                    .filter(|tag| {
                        let contains = config.tags.contains(&tag.to_lowercase());
                        if !contains {
                            if config.allow_custom_tags {
                                if tag.chars().count() as i32 > config.custom_tag_length {
                                    return false
                                }
                                return true
                            }
                            return false;
                        }
                        return contains;
                    }
                    ).map(|tag| tag.to_lowercase())
                    .collect();

                    if !sorted_tags.is_empty() {
                        safe_tags = Some(sorted_tags);
                    }
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
                    private: upload.private.unwrap_or(false),
                    tags: safe_tags
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
                        let media_id = media.id;
                        user.uploads.push(media_id.clone());

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

                        let content_id = ContentId {
                            id: media_id
                        };
                        
                        Ok(Json(content_id))
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
            None => Err(Error::Unauthorized(String::from("Api key not valid and or does not exist!")))
        }
    }

    /// Edit media information
    /// such as name, privatizing, and tags
    #[utoipa::path(
        post,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully edited media"),
            (status = 400, description = "Server received malformed client request", body = Error),
            (status = 401, description = "An authentication issue has occurred", body = Error),
            (status = 403, description = "A forbidden action has been performed by the client", body = Error),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[post("/edit?<api_key>&<id>&<name>&<private>&<tags>&<edit_tags>")]
    pub async fn edit(
        config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
        id: String,
        api_key: String,
        name: Option<String>,
        private: Option<bool>,
        tags: Option<Vec<String>>,
        edit_tags: Option<bool>,
    ) -> Result<Status, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let config = match config_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        if !config.allow_content_editing {
            return Err(Error::Forbidden(String::from("Editing is disabled on this instance")))
        }

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

        if user.is_some() {
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

            match media {
                Some(media) => {
                    let mut edited_media = media;
                    
                    if let Some(name) = name {
                        if name.len() as i32 > config.content_name_length {
                            return Err(Error::BadRequest(format!("Name length too long. Maximum of {} characters", config.content_name_length)))
                        }
                        
                        edited_media.name = name;
                    }

                    if let Some(private) = private {
                        edited_media.private = private;
                    }

                    if let Some(edit_tags) = edit_tags {
                        if edit_tags == true {
                            let mut safe_tags: Option<Vec<String>> = None;

                            if let Some(tags) = tags {
                                let sorted_tags: Vec<String> = tags
                                .into_iter()
                                .filter(|tag| {
                                    let contains = config.tags.contains(&tag.to_lowercase());
                                    if !contains {
                                        if config.allow_custom_tags {
                                            if tag.chars().count() as i32 > config.custom_tag_length {
                                                return false
                                            }
                                            return true
                                        }
                                        return false;
                                    }
                                    return contains;
                                }
                                ).map(|tag| tag.to_lowercase())
                                .collect();

                                if !sorted_tags.is_empty() {
                                    safe_tags = Some(sorted_tags);
                                }
                            }

                            edited_media.tags = safe_tags;
                        }
                    }
                    
                    match media_database.update_and_fetch(&edited_media.id, |_| {
                        Some(IVec::from(match serde_json::to_vec(&edited_media) {
                            Ok(result) => result,
                            Err(_) => return None
                        }))
                    }) {
                        Ok(_) => {
                            return Ok(Status::Ok)
                        },
                        Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
                    }
                },
                None => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
            }
        } else {
            return Err(Error::Unauthorized(String::from("Api key not valid and or does not exist!")))
        }
    }

    /// Grabs all media related tags in use on the instance
    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully grabbed all in use tags"),
            (status = 500, description = "An internal error on the server's end has occurred", body = Error)
        )
    )]
    #[get("/tags")]
    pub async fn tags(
        _config_store: &State<Arc<Mutex<Config>>>,
        database_store: &State<Arc<Mutex<sled::Db>>>,
    ) -> Result<Json<ContentTags>, Error> {
        let database = match database_store.lock() {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("Failed to access backend database")))
        };

        let media_database = match database.open_tree("media") {
            Ok(result) => result,
            Err(_) => return Err(Error::InternalError(String::from("An internal error on the server's end has occurred")))
        };

        let media_tags: Vec<String> = media_database
            .into_iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| {
                let result: DBMedia = match serde_json::from_str(&String::from_utf8_lossy(&item.1)) {
                    Ok(result) => result,
                    Err(_) => return None
                };
                Some(result)
            })
            .flat_map(|x| x.tags)
            .flatten()
            .unique()
            .collect();

        let content_tags = ContentTags {
            tags: media_tags
        };
        
        Ok(Json(content_tags))
    }
}
