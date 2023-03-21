use std::io::Bytes;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{Filter, get};
use warp::body::json;
use warp::path::Exact;

mod routes;
mod handlers;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct File {
    name: String,
    id: Uuid,
    content: String
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileSummary {
    name: String,
    id: Uuid,
    edited_time: DateTime<Utc>,
    created_time: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PreviewDetail {
    name: String,
    id: Uuid,
    r#type: PreviewDetailType,
    data: String
}
#[derive(Serialize, Deserialize, Clone)]
enum PreviewDetailType {
    PDF,
    HTML
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitCommit {
    hash: String,
    parent: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitRef {
    name: String,
    hash: String
}

//***

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileIDAndOptionalGitHash {
    id: Uuid,
    hash: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitHistory {
    commits: Vec<GitCommit>,
    refs: Vec<GitRef>
}

#[tokio::main]
async fn main() {
    let cors = warp::cors().allow_any_origin();
    let api_root = warp::path("api");

    let routes = api_root.and(routes::get_routes()).with(cors);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}