use std::fmt::format;
use std::sync::{Arc, Mutex, MutexGuard};
use log::error;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use sled::{Db, Tree};
use crate::Error;

pub trait DatabaseExtension {
    fn get_database(&self) -> Result<MutexGuard<Db>, Custom<Json<Error>>>;
}

impl DatabaseExtension for &State<Arc<Mutex<Db>>> {
    fn get_database(&self) -> Result<MutexGuard<Db>, Custom<Json<Error>>> {
        match self.lock() {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Failed to lock database, {}", err.to_string());

                Err(Custom(Status::InternalServerError, Json(Error {
                    error: String::from("An internal error on the server's end has occurred")
                })))
            }
        }
    }
}

pub trait DatabaseTreeExtension {
    fn get_tree(&self, tree_name: &str) -> Result<Tree, Custom<Json<Error>>>;
}

impl DatabaseTreeExtension for MutexGuard<'_, Db> {
    fn get_tree(&self, tree_name: &str) -> Result<Tree, Custom<Json<Error>>> {
        return match self.open_tree(tree_name) {
            Ok(result) => Ok(result),
            Err(err) => {
                error!("Failed to open database tree ({}), {}", tree_name, err.to_string());

                return Err(Custom(Status::InternalServerError, Json(Error {
                    error: String::from("An internal error on the server's end has occurred")
                })))
            }
        };
    }
}