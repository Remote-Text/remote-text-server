use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use warp::Filter;

use crate::handlers;

// Filter that limits the size of JSON files
fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

// Filter that maps to the list_files api call, then tries to fulfill the request
fn list_files() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("listFiles")
        .and_then(handlers::list_files)
}

// Filter that maps to the create_file api call, uses the json_body to restrict file size, then tries to fulfill the request
fn create_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("createFile")
        .and(json_body())
        .and_then(handlers::create_file)
}

// Filter that maps to the get_file api call, uses the json_body to restrict file size, then tries to fulfill the request
fn get_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getFile")
        .and(json_body())
        .and_then(handlers::get_file)
}

// Filter that maps to the save_file api call, uses the json_body to restrict file size, then tries to fulfill the request
fn save_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("saveFile")
        .and(json_body())
        .and_then(handlers::save_file)
}

// Filter that maps to the preview_file api call, uses the json_body to restrict file size, then tries to fulfill the request
fn preview_file() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("previewFile")
        .and(json_body())
        .and_then(handlers::preview_file)
}

// Filter that maps to the get_preview api call, uses the json_body to restrict file size, then tries to fulfill the request
fn get_preview() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getPreview")
        .and(json_body())
        .and_then(handlers::get_preview)
}

// Filter that maps to the get_history api call, uses the json_body to restrict file size, then tries to fulfill the request
fn get_history() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getHistory")
        .and(json_body())
        .and_then(handlers::get_history)
}

// Filter that contains all other relevant filters, allowing for the use of any filter through this one
pub(crate) fn get_routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_files()
        .or(create_file())
        .or(get_file())
        .or(save_file())
        .or(preview_file())
        .or(get_preview())
        .or(get_history())
}