#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply, reply};
use warp::http::StatusCode;

/* // EXTERNAL CRATE USAGE //

pretty_env_logger:
    When initialized, this crate will use the standard error log to create colorful
    and organized log outputs, making it easier to read

serde:
    A common crate that serializes rust data structures to formats such as JSON,
    and can also deserialize data into rust data structures

uuid:
    This crate provides the generation and parsing of Unique Universal Identifiers

warp:
    Based off of the popular networking crate hyper, warp provides a simple and fast
    web server framework

*/


mod routes;
mod handlers;
mod api;
mod files;
mod tests;

#[allow(non_snake_case)]
fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
#[allow(non_snake_case)]
fn PREVIEWS_DIR() -> PathBuf {
    Path::new(".").join("previews")
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    username: String,
    id: Uuid,
    password: String
}
#[derive(Serialize, Deserialize, Clone)]
struct Creds {
    username: String,
    password: String
}

#[derive(Debug)]
struct NonexistentSessionID;
impl warp::reject::Reject for NonexistentSessionID {}
async fn login(creds: Creds, users_db: Arc<Mutex<HashMap<String, User>>>, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> Box<dyn warp::Reply> {
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
async fn logout(session_id: Uuid, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> impl warp::Reply {
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

async fn is_logged_in(session_id: Uuid, session_db: Arc<Mutex<HashMap<Uuid, Uuid>>>) -> Result<Uuid, warp::Rejection> {
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

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    println!("handle_rejection: {:?}", err);
    if err.is_not_found() {
        Ok(warp::reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if let Some(e) = err.find::<NonexistentSessionID>() {
        Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR))
    }
}
#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        std::env::set_var("RUST_LOG", "remote_text_server=warn");
    }

    // Initialize pretty_env_logger so we can get organized/colorful logs
    pretty_env_logger::init();

    #[allow(non_snake_case)]
    if std::env::args().collect::<Vec<String>>().get(1) == Some(&"-vv".to_string()) {
        // log::trace!(target: "remote_text_server::main", "ENV_VARS");
        let VERGEN_BUILD_DATE = env!("VERGEN_BUILD_DATE");
        let VERGEN_BUILD_TIMESTAMP = env!("VERGEN_BUILD_TIMESTAMP");
        let VERGEN_CARGO_DEBUG = env!("VERGEN_CARGO_DEBUG");
        let VERGEN_CARGO_FEATURES = env!("VERGEN_CARGO_FEATURES");
        let VERGEN_CARGO_OPT_LEVEL = env!("VERGEN_CARGO_OPT_LEVEL");
        let VERGEN_CARGO_TARGET_TRIPLE = env!("VERGEN_CARGO_TARGET_TRIPLE");
        let VERGEN_GIT_BRANCH = env!("VERGEN_GIT_BRANCH");
        let VERGEN_GIT_COMMIT_AUTHOR_EMAIL = env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL");
        let VERGEN_GIT_COMMIT_AUTHOR_NAME = env!("VERGEN_GIT_COMMIT_AUTHOR_NAME");
        let VERGEN_GIT_COMMIT_COUNT = env!("VERGEN_GIT_COMMIT_COUNT");
        let VERGEN_GIT_COMMIT_DATE = env!("VERGEN_GIT_COMMIT_DATE");
        let VERGEN_GIT_COMMIT_MESSAGE = env!("VERGEN_GIT_COMMIT_MESSAGE");
        let VERGEN_GIT_COMMIT_TIMESTAMP = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
        let VERGEN_GIT_DESCRIBE = env!("VERGEN_GIT_DESCRIBE");
        let VERGEN_GIT_SHA = env!("VERGEN_GIT_SHA");
        let VERGEN_RUSTC_CHANNEL = env!("VERGEN_RUSTC_CHANNEL");
        let VERGEN_RUSTC_COMMIT_DATE = env!("VERGEN_RUSTC_COMMIT_DATE");
        let VERGEN_RUSTC_COMMIT_HASH = env!("VERGEN_RUSTC_COMMIT_HASH");
        let VERGEN_RUSTC_HOST_TRIPLE = env!("VERGEN_RUSTC_HOST_TRIPLE");
        let VERGEN_RUSTC_LLVM_VERSION = env!("VERGEN_RUSTC_LLVM_VERSION");
        let VERGEN_RUSTC_SEMVER = env!("VERGEN_RUSTC_SEMVER");
        let VERGEN_SYSINFO_CPU_BRAND = env!("VERGEN_SYSINFO_CPU_BRAND");
        let VERGEN_SYSINFO_CPU_CORE_COUNT = env!("VERGEN_SYSINFO_CPU_CORE_COUNT");
        let VERGEN_SYSINFO_CPU_FREQUENCY = env!("VERGEN_SYSINFO_CPU_FREQUENCY");
        let VERGEN_SYSINFO_CPU_NAME = env!("VERGEN_SYSINFO_CPU_NAME");
        let VERGEN_SYSINFO_CPU_VENDOR = env!("VERGEN_SYSINFO_CPU_VENDOR");
        let VERGEN_SYSINFO_NAME = env!("VERGEN_SYSINFO_NAME");
        let VERGEN_SYSINFO_OS_VERSION = env!("VERGEN_SYSINFO_OS_VERSION");
        let VERGEN_SYSINFO_TOTAL_MEMORY = env!("VERGEN_SYSINFO_TOTAL_MEMORY");
        let VERGEN_SYSINFO_USER = env!("VERGEN_SYSINFO_USER");

        println!("\
VERGEN_BUILD_DATE: {VERGEN_BUILD_DATE}
VERGEN_BUILD_TIMESTAMP: {VERGEN_BUILD_TIMESTAMP}
VERGEN_CARGO_DEBUG: {VERGEN_CARGO_DEBUG}
VERGEN_CARGO_FEATURES: {VERGEN_CARGO_FEATURES}
VERGEN_CARGO_OPT_LEVEL: {VERGEN_CARGO_OPT_LEVEL}
VERGEN_CARGO_TARGET_TRIPLE: {VERGEN_CARGO_TARGET_TRIPLE}
VERGEN_GIT_BRANCH: {VERGEN_GIT_BRANCH}
VERGEN_GIT_COMMIT_AUTHOR_EMAIL: {VERGEN_GIT_COMMIT_AUTHOR_EMAIL}
VERGEN_GIT_COMMIT_AUTHOR_NAME: {VERGEN_GIT_COMMIT_AUTHOR_NAME}
VERGEN_GIT_COMMIT_COUNT: {VERGEN_GIT_COMMIT_COUNT}
VERGEN_GIT_COMMIT_DATE: {VERGEN_GIT_COMMIT_DATE}
VERGEN_GIT_COMMIT_MESSAGE: {VERGEN_GIT_COMMIT_MESSAGE}
VERGEN_GIT_COMMIT_TIMESTAMP: {VERGEN_GIT_COMMIT_TIMESTAMP}
VERGEN_GIT_DESCRIBE: {VERGEN_GIT_DESCRIBE}
VERGEN_GIT_SHA: {VERGEN_GIT_SHA}
VERGEN_RUSTC_CHANNEL: {VERGEN_RUSTC_CHANNEL}
VERGEN_RUSTC_COMMIT_DATE: {VERGEN_RUSTC_COMMIT_DATE}
VERGEN_RUSTC_COMMIT_HASH: {VERGEN_RUSTC_COMMIT_HASH}
VERGEN_RUSTC_HOST_TRIPLE: {VERGEN_RUSTC_HOST_TRIPLE}
VERGEN_RUSTC_LLVM_VERSION: {VERGEN_RUSTC_LLVM_VERSION}
VERGEN_RUSTC_SEMVER: {VERGEN_RUSTC_SEMVER}
VERGEN_SYSINFO_CPU_BRAND: {VERGEN_SYSINFO_CPU_BRAND}
VERGEN_SYSINFO_CPU_CORE_COUNT: {VERGEN_SYSINFO_CPU_CORE_COUNT}
VERGEN_SYSINFO_CPU_FREQUENCY: {VERGEN_SYSINFO_CPU_FREQUENCY}
VERGEN_SYSINFO_CPU_NAME: {VERGEN_SYSINFO_CPU_NAME}
VERGEN_SYSINFO_CPU_VENDOR: {VERGEN_SYSINFO_CPU_VENDOR}
VERGEN_SYSINFO_NAME: {VERGEN_SYSINFO_NAME}
VERGEN_SYSINFO_OS_VERSION: {VERGEN_SYSINFO_OS_VERSION}
VERGEN_SYSINFO_TOTAL_MEMORY: {VERGEN_SYSINFO_TOTAL_MEMORY}
VERGEN_SYSINFO_USER: {VERGEN_SYSINFO_USER}
    ");

        std::process::exit(0);
    }

    let users = Arc::new(Mutex::new(HashMap::<String, User>::new()));
    {
        let mut users = users.lock().unwrap();
        users.insert("Sam".to_string(), User {
            username: "Sam".to_string(),
            id: Uuid::new_v4(),
            password: "password".to_string(),
        });
    }
    let users = warp::any().map(move || Arc::clone(&users));

    let sessions = Arc::new(Mutex::new(HashMap::<Uuid, Uuid>::new()));
    let sessions = warp::any().map(move || Arc::clone(&sessions));

    let login_route = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(users.clone())
        .and(sessions.clone())
        .then(login);

    // let logged_in = warp::header::<Uuid>("SESSION_ID").and_then(|uuid: Uuid| async move { if uuid.is_max() { Ok(uuid) } else { Err(warp::reject())} });
    let logged_in = warp::header::<Uuid>("SESSION_ID").and(sessions.clone()).and_then(is_logged_in);
    let hello = warp::path!("hello" / String)
        .and(logged_in.clone())
        .map(|name, uuid| format!("Hello, {}! [id: {uuid}]", name));
    let logout_route = warp::post()
        .and(warp::path("logout"))
        .and(warp::header::<Uuid>("SESSION_ID"))
        .and(sessions.clone())
        .then(logout);

    log::info!(target: "remote_text_server::main", "Searching for repositories");
    let repositories = files::repos();

    log::trace!(target: "remote_text_server::main", "Setting up filters");
    // Set up the warp wrapper with CORS (Cross-Origin Resource Sharing), allowing any origin point
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"]);
    // Sets up logging for api requests
    let log = warp::log("remote_text_server::api");
    // Sets up the root path for the api
    let api_root = warp::path("api").and(warp::path("v2"));

    log::trace!(target: "remote_text_server::main", "Setting up routes");
    // Creates a chain of filters that checks/runs each function in the API
    let routes = api_root.and(routes::get_routes(repositories.clone()))
        // .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"))
        .with(cors)
        .with(log);

    let routes = login_route.or(hello).or(logout_route);

    log::info!(target: "remote_text_server::main", "Running server");
    // Runs the server with the set up filters
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
