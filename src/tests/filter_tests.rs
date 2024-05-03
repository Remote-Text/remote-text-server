#[cfg(test)]

// #[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use futures::task::Spawn;
use serde_json;
use uuid::Uuid;
use warp::test;

use crate::{api, handlers, routes};
use crate::files::repos;
use crate::handlers::{FileIDAndGitHash, IdOnly};
use crate::routes::get_file;

#[allow(non_snake_case)]
fn FILES_DIR() -> PathBuf {
    Path::new(".").join("files")
}
#[allow(non_snake_case)]
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
    }
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

    let deserialized_result: api::CreateFileResult = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(result.status(), 200);
    assert_eq!(deserialized_result.name, "TestFile");

    clear_files_directory("test_create_files_filter", deserialized_result.id);
}

#[tokio::test]
async fn create_file_too_large() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::create_file(repositories);

    let bytes = vec![b'a' ; (1024 * 1024 * 16) + 1];
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

    let deserialized_result: api::CreateFileResult = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(deserialized_result.name, "TestFile");

    let getFileInfo = {
        let fileGitHash = repositories
            .lock()
            .unwrap();

        let fileGitHash = fileGitHash.deref().get(&deserialized_result.id).unwrap();

        let rawGitHash = fileGitHash.revparse_single("HEAD").unwrap();

        FileIDAndGitHash { id: deserialized_result.id, hash: rawGitHash.id().to_string() }
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
//      - `parent` is a Git hash, indicating the parent commit of this new commit that is being made
// - How should I find the branch name to place in FAHABN? Is that a name or also a git hash?
//      - `branch` should be something like "main". it's the name of the branch that will be forcibly updated to point to this new commit
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

    let deserialized_result: api::CreateFileResult = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(result.status(), 200);
    assert_eq!(deserialized_result.name, "TestFile");

    let getFileInfo = {
        let fileGitHash = repositories
            .lock()
            .unwrap();

        let fileGitHash = fileGitHash.deref().get(&deserialized_result.id).unwrap();

        let rawGitHash = fileGitHash.revparse_single("HEAD").unwrap();

        FileIDAndGitHash { id: deserialized_result.id, hash: rawGitHash.id().to_string() }
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

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::create_file(repositories.clone());

    let obj = handlers::NameAndOptionalContent{ name: "TestFile".to_string(), content: None };

    let result = test::request()
        .method("POST")
        .path("/createFile")
        .json(&obj)
        .reply(&filter)
        .await;

    let deserialized_result: api::CreateFileResult = serde_json::from_slice(result.body()).unwrap();

    assert_eq!(result.status(), 200);
    assert_eq!(deserialized_result.name, "TestFile");

    let filter  = routes::delete_file(repositories.clone());

    let obj = IdOnly { id: deserialized_result.id };

    let del_file_result = test::request()
        .method("POST")
        .path("/deleteFile")
        .json(&obj)
        .reply(&filter)
        .await;

    assert_eq!(del_file_result.status(), 200);
}


#[tokio::test]
async fn delete_nonexistent_repo() {

    let _ = pretty_env_logger::try_init();

    let repositories = repos();
    let filter = routes::delete_file(repositories.clone());

    let obj = IdOnly { id : Uuid::new_v4()};

    let result = test::request()
        .method("POST")
        .path("/deleteFile")
        .json(&obj)
        .reply(&filter)
        .await;

    assert_eq!(result.status(), 404);
}

