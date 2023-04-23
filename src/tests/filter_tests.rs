#[cfg(test)]

// #[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::fs;
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
use serde_json::Value::String;

use crate::{files, routes, api, handlers};
use crate::files::repos;
use crate::handlers::{FileAndHashAndBranchName, FileIDAndGitHash, IdOnly};
use crate::routes::get_file;


fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
fn PREVIEWS_DIR() -> PathBuf {
    Path::new(".").join("previews")
}


fn clear_files_directory(test_name: &str, obj_id: Uuid) {
    match fs::remove_dir_all(FILES_DIR().join(obj_id.to_string())){
      Ok(_) => {
          log::info!(target: "remote_text_server::tests", "[{}][{}] Test has finished and test files have been deleted", test_name, obj_id.to_string());
      }, Err(_) => {
            log::error!(target: "remote_text_server::tests", "[{}][{}] Test has finished, but failed to delete the test files", test_name, obj_id.to_string());
        },
    };
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

    // clear_files_directory("test_list_files_filter");
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

    clear_files_directory("test_create_files_filter", deserializedResult.id);
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

    // clear_files_directory("create_file_too_large");
}

#[tokio::test]
async fn test_get_file_filter() {

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

        let rawGitHash = fileGitHash.revparse_single("HEAD").unwrap();

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

    clear_files_directory("test_get_file_filter", obj.id);
}

// TODO: Ask Sam about the following:
// - Is the parent field from FileAndHashAndBranchName a git hash? uuid?
// - How should I find the branch name to place in FAHABN? Is that a name or also a git hash?
#[tokio::test]
#[ignore]
async fn test_save_file_filter() {

    let _ = pretty_env_logger::try_init();

    // Create a file
    let repositories = repos();
    let filter = routes::create_file(repositories.clone());

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

    let getFileInfo = {
        let fileGitHash = repositories
            .lock()
            .unwrap();

        let fileGitHash = fileGitHash.deref().get(&deserializedResult.id).unwrap();

        let rawGitHash = fileGitHash.revparse_single("HEAD").unwrap();

        FileIDAndGitHash { id: deserializedResult.id, hash: rawGitHash.id().to_string() }
    };

    // Save a new file as a child of the file we just created
    let filter = routes::save_file(repositories.clone());
    let childFileInfo = handlers::FileAndHashAndBranchName {
        name : "TestFileChild".to_string(),
        id : Uuid::new_v4(),
        content : "".to_string(),
        parent : "".to_string(),
        branch : "".to_string()
    };

    // clear_files_directory("test_save_file_filter");
}

#[tokio::test]
async fn test_delete_file_filter() {

}