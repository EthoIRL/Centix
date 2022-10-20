#[allow(non_snake_case)]
pub mod Media {
    use std::{sync::{Arc, Mutex}};

    use crate::Config::*;
    use crate::Error;

    use rocket::{
        fs::{NamedFile},
        get,
        http::Status,
        serde::json::Json,
        FromForm, State,
        FromFormField,
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct Media {
        #[schema(example = "HilrvkpJ")]
        id: String
    }

    #[derive(Serialize, Deserialize, FromForm, IntoParams, ToSchema, Clone)]
    pub struct UploadParam {
        #[schema(example = "Funny Cat")]
        name: String,
        #[schema(example = "AAAAFGZ0eXBxdCAgAAAAAHF0ICAAAAAId2lkZQADRhltZGF0AAAAHgYFGkdWStxcTEM/lO/FETzRQ6gB/8zM/wIACq5ggAAAQnUluCAC//MDYf1z+BR3n/d++1Ccx9iRJNKTtVNvQAAAAwAABhltZ6KSD1y6A1IigodKYRCnCJBE9e+vMEYutCkyDaiT+Qa2XvvNFX9WjMUNREsNnGMBdCohI32HB/TKRa/nHjyf9O9dARpyMqlmcYLUU1Uxew5pHWta5WGHqehioy6Ewcwhnl6AlkZSspDywJ9VsjqCVXFp+Rt06I51kht/ATcPxO8SOsgDIAul4emLumO+oB6QxYTsmPyXUVtCZ0CaD1yH0tB2o5UxZ3dh0RssDGHn0e9ajeS/a0aw8fGwZf95VDzPeUSybwcX4iMiLaw51G1Ozy5fZCDa1emGWBvXHWtha8Ijl+MEIgICI8eC87k3NhwR8ZEG8kOxYqrFGSaGrNPWgbv8Yv7ckkuuvXzThnUhQOabyH5otGgr+w8BnhDBKHGAToYSBc/LSGYuwqnrqxQQLwBSyM8G8SckE+/ZJHo7FJlypa4H9lLk9OZy4/3Ec1nI5VcEx3DXTbXL7swMWaPXSD1cNib4j9C5+fIsPdtypv9f5+wbD8SOYyG6i7vwwCYs0nsZgc+I+f0kgV1hqRmdgB21Zime0D9/LmT/v2l2f3SIEX4nn0hU9FomWmcLmaF+RTUxFw3uSsYIdGnVf46eAG73Yps66K5v+eTyWI/ObVySjQw0V/P7j9HJ95GS8p26Xj7xf0sqFArcpmdrkp+4LARlg2XtR5lYnIY4CNrWdj2zitLwmjZRi2GDXRcslgd/hV5PfGcTrOnSMAKCk6FGY")]
        data: String,
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
        content_type: Option<ContentType>,
    ) -> Json<Vec<Media>> {
        todo!()
    }

    //TODO: Figure out how to use mulipart file datastream with both utoipa/rocket.
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
    #[get("/upload?<upload..>")]
    pub async fn upload(
        config: &State<Arc<Mutex<Config>>>,
        database: &State<Arc<Mutex<sled::Db>>>,
        upload: UploadParam
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
