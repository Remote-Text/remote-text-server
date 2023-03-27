#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::io::Bytes;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, get};
use warp::body::json;
use warp::path::Exact;


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
mod previewing;
mod files;

#[tokio::main]
async fn main() {
    // Initialize pretty_env_logger so we can get organized/colorful logs
    pretty_env_logger::init();

    let repositories = files::repos();
    println!("REPOS");
    for (uuid, _) in repositories.lock().unwrap().iter() {
        println!("{}", uuid);
    }

    // Set up the warp wrapper with CORS (Cross-Origin Resource Sharing), allowing any origin point
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"]);
    // Sets up logging for api requests
    let log = warp::log("remote-text-server::api");
    // Sets up the root path for the api
    let api_root = warp::path("api");

    // Creates a chain of filters that checks/runs each function in the API
    let routes = api_root.and(routes::get_routes(repositories.clone()))
        // .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"))
        .with(cors)
        .with(log);

    // Runs the server with the set up filters
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}