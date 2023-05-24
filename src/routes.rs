use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use git2::Repository;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use warp::Filter;

use crate::handlers;

// Filter that limits the size of JSON files
pub(crate) fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

// Extracts the USER_ID from the optional header
pub(crate) fn user_id() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::filters::header::optional::<String>("USER_ID")
        .map(move |user_id: Option<String>| {
            let uid = user_id.unwrap_or("anonymous".to_string());
            log::trace!(target: "remote_text_server::api", "Determined effective user ID as '{uid}'");
            uid
        })
}

// Filter that maps to the list_files api call, then tries to fulfill the request
pub(crate) fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("listFiles")
        .and(user_id())
        .and_then(move |user_id| handlers::list_files(user_id, repos.clone()))
}

// Filter that maps to the create_file api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn create_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("createFile")
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and(user_id())
        .and_then(move |obj, addr, user_id| handlers::create_file(obj, addr, user_id, repos.clone()))
}

// Filter that maps to the get_file api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn get_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getFile")
        .and(json_body())
        .and(user_id())
        .and_then(move |obj, user_id| handlers::get_file(obj, user_id, repos.clone()))
}

// Filter that maps to the save_file api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn save_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("saveFile")
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and(user_id())
        .and_then(move |obj, addr, user_id| handlers::save_file(obj, addr, user_id, repos.clone()))
}

// Filter that maps to the delete_file api call, then attempts to fufill the request using handler code
pub(crate) fn delete_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("deleteFile")
        .and(user_id())
        .and(json_body())
        .and_then(move |user_id, obj| handlers::delete_file(obj, user_id, repos.clone()))
}

// Filter that maps to the preview_file api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn preview_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("previewFile")
        .and(user_id())
        .and(json_body())
        .and_then(move |user_id, obj| handlers::preview_file(obj, user_id, repos.clone()))
}

// Filter that maps to the get_preview api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn get_preview(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getPreview")
        .and(json_body())
        .and(user_id())
        .and_then(move |obj, user_id| handlers::get_preview(obj, user_id, repos.clone()))
}

// Filter that maps to the get_history api call, uses the json_body to restrict file size, then tries to fulfill the request
pub(crate) fn get_history(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getHistory")
        .and(json_body())
        .and(user_id())
        .and_then(move |obj, user_id| handlers::get_history(obj, user_id, repos.clone()))
}

// Filter that contains all other relevant filters, allowing for the use of any filter through this one
pub(crate) fn get_routes(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    list_files(repos.clone())
        .or(create_file(repos.clone()))
        .or(get_file(repos.clone()))
        .or(save_file(repos.clone()))
        .or(delete_file(repos.clone()))
        .or(preview_file(repos.clone()))
        .or(get_preview(repos.clone()))
        .or(get_history(repos.clone()))
}
