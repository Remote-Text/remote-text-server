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

use crate::api::{CompilationOutput, CompilationState, File, FileIDAndOptionalGitHash, FileSummary, GitCommit, GitHistory, GitRef, PreviewDetail, PreviewDetailType};
use crate::{files, previewing};
use crate::files::repos;

pub(crate) async fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::reply::json(&files::list_files(repos)));
    let example_files = if rand::random() {
        vec![]
    } else {
        vec![FileSummary {
            name: "README.md".to_string(),
            id: Uuid::nil(),
            edited_time: Utc::now(),
            created_time: Utc::now(),
        }, FileSummary {
            name: "main.rs".to_string(),
            id: Uuid::new_v4(),
            edited_time: Utc::now().checked_sub_days(Days::new(2)).unwrap(),
            created_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
        }]
    };
    return Ok(warp::reply::json(&example_files));
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NameAndOptionalContent {
    name: String,
    content: Option<String>
}
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

pub(crate) async fn save_file(obj: File, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
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
pub(crate) async fn get_history(file_id: IdOnly) -> Result<Box<dyn warp::Reply>, Infallible> {
    if rand::random() {
        let example_git_history = GitHistory {
            commits: vec![
                GitCommit { hash: "aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string(), parent: None },
                GitCommit { hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string(), parent: Some("aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string()) }
            ],
            refs: vec![
                GitRef { name: "main".to_string(), hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string() }
            ],
        };
        return Ok(Box::new(warp::reply::json(&example_git_history)))
    } else if rand::random() {
        let example_git_history = GitHistory {
            commits: vec![],
            refs: vec![],
        };
        return Ok(Box::new(warp::reply::json(&example_git_history)))
    } else {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    }
}