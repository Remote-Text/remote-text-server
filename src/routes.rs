use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use git2::Repository;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;

use crate::handlers;

// Filter that limits the size of JSON files
pub(crate) fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
}

// pub(crate) fn logged_in(sessions: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> impl Filter<Extract = (Uuid,), Error = warp::Rejection> + Clone {
//     let sessions = warp::any().map(move || Arc::clone(&sessions));
//     warp::header::<Uuid>("SESSION_ID").and(sessions).and_then(is_logged_in)
// }

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct User {
    pub(crate) username: String,
    pub(crate) id: Uuid,
    pub(crate) password: String
}
#[derive(Serialize, Deserialize, Clone)]
struct Creds {
    username: String,
    password: String
}

#[derive(Debug)]
struct NonexistentSessionID;
impl warp::reject::Reject for NonexistentSessionID {}
pub(crate) async fn login(creds: Creds, users_db: Arc<Mutex<HashMap<String, User>>>, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> Box<dyn warp::Reply> {
    let users = users_db.lock().unwrap();
    log::trace!(target: "remote_text_server::auth::login", "Attempting to login as user {}", &creds.username);
    match users.get(&creds.username) {
        None => {
            log::trace!(target: "remote_text_server::auth::login", "User {} does not exist", &creds.username);
            Box::new(StatusCode::UNAUTHORIZED)
        }
        Some(u) => {
            log::trace!(target: "remote_text_server::auth::login", "User {} exists; checking password", &creds.username);
            if u.password == creds.password {
                log::trace!(target: "remote_text_server::auth::login", "Password match for user {}", &creds.username);
                let session_id = Uuid::new_v4();
                log::trace!(target: "remote_text_server::auth::login", "Generated session ID {session_id} for user {} [id: {}]", &creds.username, &u.id);
                let mut sessions = session_db.lock().unwrap();
                sessions.insert(session_id, u.id);
                Box::new(session_id.to_string())
            } else {
                log::trace!(target: "remote_text_server::auth::login", "Incorrect password for user {}", &creds.username);
                Box::new(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
pub(crate) async fn logout(session_id: Uuid, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> impl warp::Reply {
    let mut db = session_db.lock().unwrap();
    log::trace!(target: "remote_text_server::auth::logout", "Attempting to remove session {session_id}");
    match db.remove(&session_id) {
        None => {
            log::trace!(target: "remote_text_server::auth::logout", "Session does not exist");
            StatusCode::UNAUTHORIZED
        },
        Some(user_id) => {
            log::trace!(target: "remote_text_server::auth::logout", "Session {session_id} (user: {user_id}) removed");
            StatusCode::OK
        }
    }
}

pub(crate) async fn is_logged_in(session_id: Uuid, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> Result<Uuid, warp::Rejection> {
    let db = session_db.lock().unwrap();
    log::trace!(target: "remote_text_server::auth::is_logged_in", "Checking session {session_id}");
    match db.get(&session_id) {
        None => {
            log::trace!(target: "remote_text_server::auth::is_logged_in", "Session {session_id} does not exist");
            Err(warp::reject::custom(NonexistentSessionID))
        },
        Some(u) => {
            log::trace!(target: "remote_text_server::auth::is_logged_in", "Session {session_id} is valid and corresponds to user {u}");
            Ok(*u)
        }
    }
}

// Filter that contains all other relevant filters, allowing for the use of any filter through this one
pub(crate) fn get_routes(repos: Arc<Mutex<HashMap<Uuid, Repository>>>, users: Arc<Mutex<HashMap<String, User>>>, sessions: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let users = warp::any().map(move || Arc::clone(&users));
    let sessions = warp::any().map(move || Arc::clone(&sessions));

    let login_route = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(users.clone())
        .and(sessions.clone())
        .then(login);
    let logged_in = warp::header::<Uuid>("SESSION_ID").and(sessions.clone()).and_then(is_logged_in);
    let logout_route = warp::post()
        .and(warp::path("logout"))
        .and(warp::header::<Uuid>("SESSION_ID"))
        .and(sessions.clone())
        .then(logout);

    let copy = repos.clone();
    let list_files = warp::path("listFiles")
        .and(logged_in.clone())
        .and_then(move |u| handlers::list_files(copy.clone()));
    let copy = repos.clone();
    let create_file = warp::path("createFile")
        .and(logged_in.clone())
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and_then(move |u, obj, addr| handlers::create_file(obj, addr, copy.clone()));
    let copy = repos.clone();
    let get_file = warp::path("getFile")
        .and(logged_in.clone())
        .and(json_body())
        .and_then(move |u, obj| handlers::get_file(obj, copy.clone()));
    let copy = repos.clone();
    let save_file = warp::path("saveFile")
        .and(logged_in.clone())
        .and(json_body())
        .and(warp::filters::addr::remote())
        .and_then(move |u, obj, addr| handlers::save_file(obj, addr, copy.clone()));
    let copy = repos.clone();
    let delete_file = warp::path("deleteFile")
        .and(logged_in.clone())
        .and(json_body())
        .and_then(move |u, obj| handlers::delete_file(obj, copy.clone()));
    let copy = repos.clone();
    let preview_file = warp::path("previewFile")
        .and(logged_in.clone())
        .and(json_body())
        .and_then(move |u, obj| handlers::preview_file(obj, copy.clone()));
    let copy = repos.clone();
    let get_preview = warp::path("getPreview")
        .and(logged_in.clone())
        .and(json_body())
        .and_then(move |u, obj| handlers::get_preview(obj, copy.clone()));
    let copy = repos.clone();
    let get_history = warp::path("getHistory")
        .and(logged_in.clone())
        .and(json_body())
        .and_then(move |u, obj| handlers::get_history(obj, copy.clone()));

    login_route
        .or(logout_route)
        .or(list_files)
        .or(create_file)
        .or(get_file)
        .or(save_file)
        .or(delete_file)
        .or(preview_file)
        .or(get_preview)
        .or(get_history)
}
