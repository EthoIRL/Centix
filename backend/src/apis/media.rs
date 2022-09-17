#[allow(non_snake_case)]
pub mod Media {
    use std::sync::{Arc, Mutex};

    use crate::Config::*;
    use rocket::{
        fs::{NamedFile, TempFile},
        get,
        http::Status,
        serde::json::Json,
        FromForm, State,
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct Media {
        #[schema(example = "aBcdEfgH")]
        id: String,
    }

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct UploadParam {
        #[schema(example = "Cool file name")]
        name: String,
        #[schema(example = "Publicly listed on /all/ endpoint")]
        private: Option<bool>,
    }

    #[utoipa::path(
        get,
        responses(
            (status = 200, description = "Successfully found media")
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
        content_type: Option<String>,
    ) -> Json<Vec<Media>> {
        todo!()
    }

    #[utoipa::path(
        get,
        context_path = "/media",
        responses(
            (status = 200, description = "Successfully uploaded media")
        ),
        params(
            UploadParam
        )
    )]
    #[get("/upload?<upload..>", format = "plain", data = "<file>")]
    pub async fn upload(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        upload: UploadParam,
        mut file: TempFile<'_>,
    ) -> Status {
        todo!()
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
