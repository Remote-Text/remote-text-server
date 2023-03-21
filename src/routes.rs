use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use warp::Filter;

use crate::handlers;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Hello {
    pub(crate) hello: String
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Goodbye {
    pub(crate) goodbye: String
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

fn list_files() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("listFiles")
        .and_then(handlers::list_files)
}

fn create_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("createFile")
        .and(json_body())
        .and_then(handlers::create_file)
}

fn get_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getFile")
        .and(json_body())
        .and_then(handlers::get_file)
}

fn save_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("saveFile")
        .and(json_body())
        .and_then(handlers::save_file)
}

fn preview_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("previewFile")
        .and(json_body())
        .and_then(handlers::preview_file)
}

fn get_preview() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getPreview")
        .and(json_body())
        .and_then(handlers::get_preview)
}

fn get_history() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getHistory")
        .and(json_body())
        .and_then(handlers::get_history)
}

pub(crate) fn get_routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_files()
        .or(create_file())
        .or(get_file())
        .or(save_file())
        .or(preview_file())
        .or(get_preview())
        .or(get_history())
}