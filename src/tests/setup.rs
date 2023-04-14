// #[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::io::Bytes;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use futures::task::Spawn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{cors, Filter, get};
use warp::body::json;
use warp::path::Exact;
use warp::test;

use crate::{files, routes, api};
use crate::files::repos;
use crate::routes::{get_routes, list_files};


fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
fn PREVIEWS_DIR() -> PathBuf {
    Path::new(".").join("previews")
}


// In the setup file, we spin up an instance of the server for test threads to send requests to

// #[tokio::test]
// async fn test_instance() {
//     // Initialize pretty_env_logger so we can get organized/colorful logs
//     pretty_env_logger::init();
//
//     log::info!(target: "remote_text_server::main", "Searching for repositories");
//     let repositories = files::repos();
//
//     log::trace!(target: "remote_text_server::main", "Setting up filters");
//     // Set up the warp wrapper with CORS (Cross-Origin Resource Sharing), allowing any origin point
//     let cors = warp::cors()
//         .allow_any_origin()
//         .allow_header("content-type")
//         .allow_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"]);
//     // Sets up logging for api requests
//     let log = warp::log("remote_text_server::api");
//     // Sets up the root path for the api
//     let api_root = warp::path("api");
//
//     log::trace!(target: "remote_text_server::main", "Setting up routes");
//     // Creates a chain of filters that checks/runs each function in the API
//     let routes = api_root.and(routes::get_routes(repositories.clone()))
//         // .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"))
//         .with(cors)
//         .with(log);
//
//     log::trace!(target: "remote_text_server::main", "Running server");
//     // Runs the server with the set up filters
//     warp::serve(routes)
//         .run(([0, 0, 0, 0], 3030))
//         .await;
// }


#[tokio::test]
async fn test_list_files_filter() {
    let repositories = files::repos();
    let filter = list_files(repositories);

    let result = warp::test::request()
        .method("POST")
        .path("listFiles")
        .reply(&filter)
        .await;

    // assert_eq!(result.status(), 200);

}