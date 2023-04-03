use std::any::Any;
use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::io::Write;
use std::net::SocketAddr;
use std::ops::Index;
use std::path::Path;
use std::sync::{Arc, Mutex};

use base64::{engine, Engine};
use chrono::{DateTime, Days, Utc};
use git2::{IndexAddOption, Repository, Signature, Time};
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

TODO: Comment create_file() functionality & general description

*/
pub(crate) async fn create_file(name: NameAndOptionalContent, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<impl warp::Reply, Infallible> {
    let now = Utc::now();
    let uuid = Uuid::new_v4();
    let Ok(repo) = Repository::init(uuid.to_string()) else {
        panic!();
    };
    let time = Time::new(now.timestamp(), 0);
    let them = if addr.is_some() {
        addr.unwrap().to_string()
    } else {
        "".to_string()
    };
    let fp = Path::new(uuid.to_string().as_str()).join(&name.name);
    let Ok(mut file) = std::fs::File::create(fp) else {
        panic!("Unable to create file!")
    };
    if let Some(content) = name.content {
        file.write_all(content.as_ref()).unwrap();
    }
    let their_sig = Signature::new(&them, "blinky@remote-text.com", &time).unwrap();
    let our_sig = Signature::new("Remote Text", "blinky@remote-text.com", &time).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(&["."], IndexAddOption::DEFAULT, None).unwrap();
    index.write();
    let tree_id = index.write_tree().unwrap();
    let co = repo.commit(Some("HEAD"), &their_sig, &our_sig, "", &repo.find_tree(tree_id).unwrap(), &vec![]).unwrap();
    println!("{}", co);
    let example_file = FileSummary {
        name: name.name,
        id: uuid,
        edited_time: now,
        created_time: now,
    };
    repos.lock().unwrap().insert(uuid, repo);
    return Ok(warp::reply::json(&example_file));
}

/*
// GET FILE //

TODO: Comment get_file() functionality & general description

*/
pub(crate) async fn get_file(obj: FileIDAndOptionalGitHash, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&obj.id) else {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    if let Some(hash) = obj.hash {
        repo.set_head(hash.as_str()).unwrap();
    }
    // repo.set_head(obj.hash.unwrap_or("HEAD".to_string()).as_str()).unwrap();
    repo.checkout_head(None).unwrap();
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
                    return Ok(Box::new(warp::reply::json(&File {
                        name: fname.clone().into_string().unwrap(),
                        id: obj.id,
                        content: content.to_string(),
                    })));
                } else {
                    return Ok(Box::new(warp::reply::json(&File {
                        name: "".to_string(),
                        id: obj.id,
                        content: "".to_string(),
                    })));
                }
            } else {
                eprintln!("Cannot read repo dir for UUID {}", obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            }
        } else {
            eprintln!("Parent to git dir does not exist for UUID {}", obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        }
    } else {
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

*/
pub(crate) async fn save_file(obj: FileAndHashAndBranchName, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&obj.id) else {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    let Some(path) = repo.path().parent() else {
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    if !path.exists() {
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
                println!("Removed {:?}", path);
                // index.add_path("../")
                // index.add_path(path.strip_prefix(cd.as_path()).unwrap());
            } else {
                eprintln!("Unable to remove {:?}", path);
            }
        }
    }
    let file_path = path.join(obj.name);
    if std::fs::write(file_path, obj.content).is_err() {
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    }

    //Perform commit
    let par = repo.head().unwrap().peel_to_commit().unwrap();
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
    let co = repo.commit(Some("HEAD"), &their_sig, &our_sig, "", &repo.find_tree(tree_id).unwrap(), &[&par]).unwrap();

    let gc = GitCommit {
        hash: co.to_string(),
        parent: Some(par.id().to_string()),
    };
    return Ok(Box::new(warp::reply::json(&gc)));
}

/*
// PREVIEW FILE //


TODO: Comment preview_file() functionality & general description

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
    let repos = repos.lock().unwrap();
    let Some(repo) = repos.get(&file_id.id) else {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    };
    let odb = repo.odb().unwrap();
    let mut commits = vec![];
    odb.foreach(|oid| {
        let Ok(commit) = repo.find_commit(*oid) else {
            return true;
        };
        let parent = commit.parent_ids().next().map(|cm| cm.to_string());
        // let parent = commit.parent(1).ok().map(|cm| cm.id().to_string());
        commits.push(GitCommit { hash: commit.id().to_string(), parent });
        true
    }).unwrap();
    // repo.references().iter().next().unwrap().
    // for _ref in repo.references().iter() {
    //     _ref
    // }
    let refs = repo.branches(None).unwrap().map(|b| {
        let (branch, branch_type) = b.unwrap();
        let name = branch.name().unwrap().unwrap().to_string();
        let hash = branch.get().peel_to_commit().unwrap().id().to_string();
        return GitRef {
            name,
            hash,
        }
    }).collect::<Vec<GitRef>>();
    let history = GitHistory {
        commits,
        refs,
    };
    return Ok(Box::new(warp::reply::json(&history)))
}