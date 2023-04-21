#[cfg(test)]

// #[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::io::Bytes;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::id;
use std::str::Utf8Error;

use chrono::{DateTime, Utc};
use futures::task::Spawn;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use warp::{cors, Filter, get};
use warp::body::json;
use warp::path::Exact;
use warp::test;
use serde_json;

use crate::{files, routes, api, handlers};
use crate::files::repos;
use crate::handlers::FileIDAndGitHash;
use crate::routes::get_file;


fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
fn PREVIEWS_DIR() -> PathBuf {
    Path::new(".").join("previews")
}


#[tokio::test]
async fn test_list_files_filter() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::list_files(repositories);

    let result = test::request()
        .method("POST")
        .path("/listFiles")
        .reply(&filter)
        .await;

    assert_eq!(result.status(), 200);
}


#[tokio::test]
async fn test_create_file_filter() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::create_file(repositories);

    let obj = handlers::NameAndOptionalContent{ name: "TestFile".to_string(), content: None };

    let result = test::request()
        .method("POST")
        .path("/createFile")
        .json(&obj)
        .reply(&filter)
        .await;

    let deserializedResult : api::FileSummary = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(result.status(), 200);
    assert_eq!(deserializedResult.name, "TestFile");
}

#[tokio::test]
async fn create_file_too_large() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::create_file(repositories);

    let bytes = vec![b'a' ; (1024 * 16) + 1];
    let body  = std::str::from_utf8(&bytes).unwrap();
    let body = format!("{}", body);
    let body = Some(body);
    let obj = handlers::NameAndOptionalContent{ name: "TestFile".to_string(), content: body };

    let result = test::request()
        .method("POST")
        .path("/createFile")
        .json(&obj)
        .reply(&filter)
        .await;

    assert_eq!(result.status(), 413);
}

#[tokio::test]
async fn test_get_file() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();

    // Finding a file requires both an ID and GitHash

    // First we create a file
    let obj = handlers::NameAndOptionalContent{ name: "TestFile".to_string(), content: None };
    let filter = routes::create_file(repositories.clone());


    let result = test::request()
        .method("POST")
        .path("/createFile")
        .json(&obj)
        .reply(&filter)
        .await;

    assert_eq!(result.status(), 200);

    let deserializedResult : api::FileSummary = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(deserializedResult.name, "TestFile");

    let getFileInfo = {
        let fileGitHash = repositories
            .lock()
            .unwrap();

        let fileGitHash = fileGitHash.deref().get(&deserializedResult.id).unwrap();

        let rawGitHash = "HEAD";
        let rawGitHash = fileGitHash.revparse_single(rawGitHash).unwrap();

        FileIDAndGitHash { id: deserializedResult.id, hash: rawGitHash.id().to_string() }
    };



    let filter = get_file(repositories.clone());
    let obj = getFileInfo;

    let result = test::request()
        .method("POST")
        .path("/getFile")
        .json(&obj)
        .reply(&filter)
        .await;

    assert_eq!(result.status(), 200);
}