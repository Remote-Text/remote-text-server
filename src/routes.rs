use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use git2::Repository;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use warp::Filter;

use crate::handlers;

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("listFiles")
        .and_then(move || handlers::list_files(repos.clone()))
}

fn create_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("createFile")
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and_then(move |obj, addr| handlers::create_file(obj, addr, repos.clone()))
}

fn get_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getFile")
        .and(json_body())
        .and_then(move |obj| handlers::get_file(obj, repos.clone()))
}

fn save_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("saveFile")
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and_then(move |obj, addr| handlers::save_file(obj, addr, repos.clone()))
}

fn delete_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("deleteFile")
        .and(json_body())
        .and_then(move |obj| handlers::delete_file(obj, repos.clone()))
}

fn preview_file(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("previewFile")
        .and(json_body())
        .and_then(move |obj| handlers::preview_file(obj, repos.clone()))
}

fn get_preview(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getPreview")
        .and(json_body())
        .and_then(handlers::get_preview)
}

fn get_history(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("getHistory")
        .and(json_body())
        .and_then(move |obj| handlers::get_history(obj, repos.clone()))
}

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