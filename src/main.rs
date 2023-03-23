#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::io::Bytes;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, get};
use warp::body::json;
use warp::path::Exact;

mod routes;
mod handlers;
mod api;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cors = warp::cors().allow_any_origin();
    let log = warp::log("remote-text-server::api");
    let api_root = warp::path("api");

    let routes = api_root.and(routes::get_routes())
        .with(cors)
        .with(log);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}