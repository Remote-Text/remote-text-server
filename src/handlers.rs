use std::any::Any;
use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::io::Write;
use std::net::SocketAddr;
use std::ops::Index;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::fs;
use std::fs::remove_dir_all;

use base64::{engine, Engine};
use chrono::{DateTime, Days, Utc};
use git2::{IndexAddOption, Oid, Repository, Signature, StatusShow, Time};
use git2::build::CheckoutBuilder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::fs::file;
use warp::hyper::StatusCode;

use crate::{files, previewing};
use crate::api::{CompilationOutput, CompilationState, File, FileIDAndOptionalGitHash, FileSummary, GitCommit, GitHistory, GitRef, PreviewDetail, PreviewDetailType};
use crate::files::repos;

pub(crate) async fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::reply::json(&files::list_files(repos)));
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NameAndOptionalContent {
    name: String,
    content: Option<String>
}

/*
// CREATE FILE //
This function will take the file name (and optional content), an address, and a corresponding
repository (?) to create a new file instance, as well as start its git history.

// TODO: Comment create_file() functionality & general description
// TODO: Make files save to a designated directory

*/
pub(crate) async fn create_file(name: NameAndOptionalContent, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<impl warp::Reply, Infallible> {
    let now = Utc::now();
    let uuid = Uuid::new_v4();
    log::info!(target: "remote_text_server::create_file", "[{}] Creating new file", uuid);
    let Ok(repo) = Repository::init(uuid.to_string()) else {
        log::error!(target: "remote_text_server::create_file", "[{}] Cannot create repository", uuid);
        panic!();
    };
    let time = Time::new(now.timestamp(), 0);
    let them = if addr.is_some() {
        addr.unwrap().to_string()
    } else {
        log::warn!(target: "remote_text_server::create_file", "[{}] Non-socket connection", uuid);
        "Non Socket Remote User".to_string()
    };
    let fp = Path::new(uuid.to_string().as_str()).join(&name.name);
    let Ok(mut file) = std::fs::File::create(fp) else {
        log::error!(target: "remote_text_server::create_file", "[{}] Unable to create file", uuid);
        panic!("Unable to create file!")
    };
    if let Some(content) = name.content {
        log::trace!(target: "remote_text_server::create_file", "[{}] Writing initial content to file", uuid);
        file.write_all(content.as_ref()).unwrap();
    }
    let their_sig = Signature::new(&them, "blinky@remote-text.com", &time).unwrap();
    let our_sig = Signature::new("Remote Text", "blinky@remote-text.com", &time).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(&["."], IndexAddOption::DEFAULT, None).unwrap();
    index.write();
    let tree_id = index.write_tree().unwrap();
    let co = repo.commit(Some("HEAD"), &their_sig, &our_sig, "", &repo.find_tree(tree_id).unwrap(), &vec![]).unwrap();
    log::info!(target: "remote_text_server::create_file", "[{}] Made initial commit ({})", uuid, co.to_string());
    let example_file = FileSummary {
        name: name.name,
        id: uuid,
        edited_time: now,
        created_time: now,
    };
    log::trace!(target: "remote_text_server::create_file", "[{}] Inserting new repo into hash map", uuid);
    repos.lock().unwrap().insert(uuid, repo);
    log::trace!(target: "remote_text_server::create_file", "[{}] Inserted new repo into hash map", uuid);
    return Ok(warp::reply::json(&example_file));
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileIDAndGitHash {
    pub(crate) id: Uuid,
    pub(crate) hash: String
}

/*
// GET FILE //

TODO: Comment get_file() functionality & general description

*/
pub(crate) async fn get_file(obj: FileIDAndGitHash, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    log::trace!(target: "remote_text_server::get_file", "[{}] Acquiring lock on hash map", &obj.id);
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&obj.id) else {
        log::info!(target: "remote_text_server::get_file", "[{}] Request made to get nonexistent file", &obj.id);
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    /*
    repo.set_head(obj.hash.as_str()).unwrap();
     */
    let Ok(oid) = Oid::from_str(obj.hash.as_str()) else {
        log::error!(target: "remote_text_server::get_file", "[{}] Cannot construct OID from hash {}", &obj.id, obj.hash);
        return Ok(Box::new(StatusCode::BAD_REQUEST));
    };
    log::trace!(target: "remote_text_server::get_file", "[{}] Setting HEAD to {}", &obj.id, oid.to_string());
    let Ok(_) = repo.set_head_detached(oid) else {
        //The hash we were given does not exist
        log::info!(target: "remote_text_server::get_file", "[{}] Unable to set HEAD (invalid hash)", &obj.id);
        return Ok(Box::new(StatusCode::BAD_REQUEST));
    };
    log::trace!(target: "remote_text_server::get_file", "[{}] Set HEAD", &obj.id);
    // repo.checkout_head(Some(CheckoutBuilder::new().force())).unwrap();
    log::trace!(target: "remote_text_server::get_file", "[{}] Checking out HEAD", &obj.id);
    let Ok(_) = repo.checkout_head(None) else {
        log::error!(target: "remote_text_server::get_file", "[{}] Unable to checkout", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    if std::path::Path::new(repo.path()).exists() {
        if let Some(path) = repo.path().parent() {
            if let Ok(entries) = std::fs::read_dir(path) {
                if let Some((fname, content)) = entries.into_iter()
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let file_type = entry.file_type().ok();
                        Some((entry, file_type?))
                    })
                    // .filter_map(|entry| Some((entry, entry.file_type().ok()?)))
                    .filter(|(_, file_type)| file_type.is_file())
                    .filter_map(|(entry, _)| {
                        Some((entry.file_name(), std::fs::read_to_string(entry.path()).ok()?))
                    })
                    .collect::<Vec<_>>()
                    .first() {
                    let Ok(filename) = fname.clone().into_string() else {
                        log::error!(target: "remote_text_server::get_file", "[{}] Cannot convert filename '{:?}' to string", &obj.id, fname.clone());
                        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
                    };
                    log::info!(target: "remote_text_server::get_file", "[{}] Found file {}", &obj.id, filename);
                    return Ok(Box::new(warp::reply::json(&File {
                        name: filename,
                        id: obj.id,
                        content: content.to_string(),
                    })));
                } else {
                    log::error!(target: "remote_text_server::get_file", "[{}] No file found in repo", &obj.id);
                    return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
                    // return Ok(Box::new(warp::reply::json(&File {
                    //     name: "".to_string(),
                    //     id: obj.id,
                    //     content: "".to_string(),
                    // })));
                }
            } else {
                log::error!(target: "remote_text_server::get_file", "[{}] Cannot read repo dir", &obj.id);
                eprintln!("Cannot read repo dir for UUID {}", obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            }
        } else {
            log::error!(target: "remote_text_server::get_file", "[{}] Parent to git dir does not exist", &obj.id);
            eprintln!("Parent to git dir does not exist for UUID {}", obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        }
    } else {
        log::error!(target: "remote_text_server::get_file", "[{}] No repo exists", &obj.id);
        eprintln!("No repo exists for UUID {}", obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileAndHashAndBranchName {
    name: String,
    id: Uuid,
    content: String,
    parent: String,
    branch: String
}

/*
// SAVE FILE //

TODO: Comment save_file() functionality & general description
//TODO: make commit off of parent
//TODO: update branch to point to new commit

*/
pub(crate) async fn save_file(obj: FileAndHashAndBranchName, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    log::trace!(target: "remote_text_server::save_file", "[{}] Acquiring lock on hash map", &obj.id);
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&obj.id) else {
        log::info!(target: "remote_text_server::save_file", "[{}] Request made to save nonexistent file", &obj.id);
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    let Some(path) = repo.path().parent() else {
        log::error!(target: "remote_text_server::save_file", "[{}] Parent to git dir cannot be found", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    if !path.exists() {
        log::trace!(target: "remote_text_server::save_file", "[{}] Parent to git dir does not exist", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        let cd = std::env::current_dir().unwrap();
        for path in entries.into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok();
                Some((entry, file_type?))
            })
            // .filter_map(|entry| Some((entry, entry.file_type().ok()?)))
            .filter(|(_, file_type)| file_type.is_file())
            .map(|(entry, _)| entry.path())
            .into_iter() {
            if std::fs::remove_file(path.clone()).is_ok() {
                log::trace!(target: "remote_text_server::save_file", "[{}] Removed {:?}", &obj.id, path);
                // index.add_path("../")
                // index.add_path(path.strip_prefix(cd.as_path()).unwrap());
            } else {
                log::error!(target: "remote_text_server::save_file", "[{}] Unable to remove {:?}", &obj.id, path);
            }
        }
    } else {
        log::error!(target: "remote_text_server::save_file", "[{}] No files found in repo", &obj.id);
    }
    let file_path = path.join(&obj.name);
    if std::fs::write(file_path, obj.content).is_err() {
        log::error!(target: "remote_text_server::save_file", "[{}] Unable to write to file", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    }
    log::trace!(target: "remote_text_server::save_file", "[{}] Wrote content to {}", &obj.id, &obj.name);

    //Perform commit
    let Ok(parent_oid) = Oid::from_str(obj.parent.as_str()) else {
        log::error!(target: "remote_text_server::save_file", "[{}] Parent is not a valid git hash ({})", &obj.id, obj.parent);
        return Ok(Box::new(StatusCode::BAD_REQUEST));
    };
    let Ok(par) = repo.find_commit(parent_oid) else {
        log::error!(target: "remote_text_server::save_file", "[{}] Unable to locate parent commit", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    log::trace!(target: "remote_text_server::save_file", "[{}] Located parent commit ({})", &obj.id, par.id().to_string());
    log::trace!(target: "remote_text_server::save_file", "[{}] Detaching head", &obj.id);
    repo.set_head_detached(parent_oid).unwrap();
    log::trace!(target: "remote_text_server::save_file", "[{}] Detached head", &obj.id);
    log::trace!(target: "remote_text_server::save_file", "[{}] Creating branch pointing to parent commit ({})", &obj.id, obj.branch);
    repo.branch(obj.branch.as_str(), &par, true).unwrap();
    log::trace!(target: "remote_text_server::save_file", "[{}] Created branch", &obj.id);
    let now = Utc::now();
    let time = Time::new(now.timestamp(), 0);
    let them = if addr.is_some() {
        addr.unwrap().to_string()
    } else {
        "".to_string()
    };
    let their_sig = Signature::new(&them, "blinky@remote-text.com", &time).unwrap();
    let our_sig = Signature::new("Remote Text", "blinky@remote-text.com", &time).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(&["."], IndexAddOption::DEFAULT, None).unwrap();
    index.write();
    let tree_id = index.write_tree().unwrap();
    log::trace!(target: "remote_text_server::save_file", "[{}] Making commit", &obj.id);
    let co = repo.commit(Some(format!("refs/heads/{}", obj.branch).as_str()), &their_sig, &our_sig, "", &repo.find_tree(tree_id).unwrap(), &[&par]).unwrap();
    log::trace!(target: "remote_text_server::save_file", "[{}] Made commit ({})", &obj.id, co.to_string());
    log::trace!(target: "remote_text_server::save_file", "[{}] Checking out new commit", &obj.id);
    repo.set_head(format!("refs/heads/{}", obj.branch).as_str()).unwrap();
    log::trace!(target: "remote_text_server::save_file", "[{}] Checked out new commit", &obj.id);

    let gc = GitCommit {
        hash: co.to_string(),
        parent: Some(par.id().to_string()),
    };
    return Ok(Box::new(warp::reply::json(&gc)));
}

/*
// DELETE FILE //

*/
pub(crate) async fn delete_file(obj: IdOnly, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    log::trace!(target: "remote_text_server::delete_file", "[{}] Acquiring lock on hash map", &obj.id);
    let mut repos = repos.lock().unwrap();

    // 1. See if repo exists
    let Some(_) = repos.get(&obj.id) else {
        log::info!(target: "remote_text_server::delete_file", "[{}] Request made to delete nonexistent file", &obj.id);
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };

    // 2. Delete repo rust object
    // // First delete the repo object from the hash map
    repos.remove(&obj.id);
    log::info!(target: "remote_text_server::delete_file", "[{}] Target repo deleted", &obj.id);

    // 3. Delete file on disk
    let uuid_string = &obj.id.to_string();
    match remove_dir_all(format!("./{uuid_string}")) {
        Ok(_) => {
            log::info!(target: "remote_text_server::delete_file", "[{}] Target directory successfully removed", &obj.id);
            return Ok(Box::new(StatusCode::OK))
        },
        Err(_) => {
            log::info!(target: "remote_text_server::delete_file", "[{}] Target directory was unable to be removed", &obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/*
// PREVIEW FILE //


TODO: Comment preview_file() functionality & general description
TODO: do

*/
pub(crate) async fn preview_file(obj: FileIDAndOptionalGitHash) -> Result<Box<dyn warp::Reply>, Infallible> {
    return if rand::random() {
        Ok(Box::new(warp::reply::json(&CompilationOutput {
            state: CompilationState::SUCCESS,
            log: "".to_string(),
        })))
    } else if rand::random() {
        Ok(Box::new(warp::reply::json(&CompilationOutput {
            state: CompilationState::FAILURE,
            log: "".to_string(),
        })))
    } else {
        Ok(Box::new(StatusCode::NOT_FOUND))
    };
}

/*
// GET PREVIEW //


TODO: Comment get_preview() functionality & general description
TODO: do

*/
pub(crate) async fn get_preview(obj: FileIDAndOptionalGitHash) -> Result<Box<dyn warp::Reply>, Infallible> {
    // if files::file_exists(obj.id) {
    //
    //     Ok(Box::new(previewing::get_preview(obj.id, obj.hash.unwrap_or("HEAD".to_string()))))
    // } else {
    //     // Even the raw file doesn't exist
        Ok(Box::new(StatusCode::NOT_FOUND))
    // }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct IdOnly {
    id: Uuid
}

/*
// GET HISTORY //

TODO: Comment get_history() functionality & general description

*/
pub(crate) async fn get_history(file_id: IdOnly, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    log::trace!(target: "remote_text_server::get_history", "[{}] Acquiring lock on hash map", &file_id.id);
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&file_id.id) else {
        log::info!(target: "remote_text_server::get_history", "[{}] Request made to get history of nonexistent file", &file_id.id);
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    let odb = repo.odb().unwrap();
    log::trace!(target: "remote_text_server::get_history", "[{}] Opened object database", &file_id.id);
    let mut commits = vec![];
    odb.foreach(|oid| {
        log::trace!(target: "remote_text_server::get_history", "[{}] Object {} located in database", &file_id.id, oid.to_string());
        let Ok(commit) = repo.find_commit(*oid) else {
            log::trace!(target: "remote_text_server::get_history", "[{}] Object {} is not commit", &file_id.id, oid.to_string());
            return true;
        };
        let parent = commit.parent_ids().next().map(|cm| cm.to_string());
        log::trace!(target: "remote_text_server::get_history", "[{}] Parent of commit {} is {:?}", &file_id.id, oid.to_string(), parent);
        // let parent = commit.parent(1).ok().map(|cm| cm.id().to_string());
        commits.push(GitCommit { hash: commit.id().to_string(), parent });
        true
    }).unwrap();
    // repo.references().iter().next().unwrap().
    // for _ref in repo.references().iter() {
    //     _ref
    // }
    log::trace!(target: "remote_text_server::get_history", "[{}] Iterating through branches", &file_id.id);
    let refs = repo.branches(None).unwrap().map(|b| {
        log::trace!(target: "remote_text_server::get_history", "[{}] Investigating branch", &file_id.id);
        let (branch, branch_type) = b.unwrap();
        let name = branch.name().unwrap().unwrap().to_string();
        log::trace!(target: "remote_text_server::get_history", "[{}] Branch name: {}", &file_id.id, name);
        let hash = branch.get().peel_to_commit().unwrap().id().to_string();
        log::trace!(target: "remote_text_server::get_history", "[{}] Branch ref: {}", &file_id.id, hash);
        return GitRef {
            name,
            hash,
        }
    }).collect::<Vec<GitRef>>();
    log::info!(target: "remote_text_server::get_history", "[{}] History loaded", &file_id.id);
    let history = GitHistory {
        commits,
        refs,
    };
    return Ok(Box::new(warp::reply::json(&history)))
}
