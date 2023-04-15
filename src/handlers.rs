use std::any::Any;
use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::fs;
use std::hash::Hash;
use std::io::Write;
use std::net::SocketAddr;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};

use base64::{engine, Engine};
use chrono::{DateTime, Days, Utc};
use git2::{IndexAddOption, Oid, Repository, Signature, StatusShow, Time};
use git2::build::CheckoutBuilder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::fs::file;
use warp::http::HeaderValue;
use warp::hyper::{Body, StatusCode};
use warp::reply::Response;

use crate::{files, FILES_DIR, PREVIEWS_DIR};
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
    let Ok(repo) = Repository::init(FILES_DIR().join(uuid.to_string())) else {
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
    let fp = Path::new(".").join("files").join(uuid.to_string()).join(&name.name);
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
    log::trace!(target: "remote_text_server::get_file", "[{}] Calling get_file_contents", &obj.id);
    return Ok(match get_file_contents(&obj.id, &obj.hash, &repos) {
        Ok((filename, content)) => {
            log::trace!(target: "remote_text_server::get_file", "[{}] Located filename and content", &obj.id);
            Box::new(warp::reply::json(&File {
                name: filename,
                id: obj.id,
                content,
            }))
        },
        Err(code) => {
            log::trace!(target: "remote_text_server::get_file", "[{}] Unable to locate file", &obj.id);
            Box::new(code)
        }
    })
}

fn get_file_contents(uuid: &Uuid, hash: &String, repos: &MutexGuard<HashMap<Uuid, Repository>>) -> Result<(String, String), StatusCode> {
    let Some(repo) = repos.get(&uuid) else {
        log::info!(target: "remote_text_server::get_file_contents", "[{}] Request made to get nonexistent file", &uuid);
        return Err(StatusCode::NOT_FOUND);
    };
    /*
    repo.set_head(obj.hash.as_str()).unwrap();
     */
    let Ok(oid) = Oid::from_str(hash.as_str()) else {
        log::info!(target: "remote_text_server::get_file_contents", "[{}] Cannot construct OID from hash {}", &uuid, &hash);
        return Err(StatusCode::BAD_REQUEST);
    };
    log::trace!(target: "remote_text_server::get_file_contents", "[{}] Setting HEAD to {}", &uuid, oid.to_string());
    let Ok(_) = repo.set_head_detached(oid) else {
        //The hash we were given does not exist
        log::info!(target: "remote_text_server::get_file_contents", "[{}] Unable to set HEAD (invalid hash)", &uuid);
        return Err(StatusCode::BAD_REQUEST);
    };
    log::trace!(target: "remote_text_server::get_file_contents", "[{}] Set HEAD", &uuid);
    log::trace!(target: "remote_text_server::get_file_contents", "[{}] Checking out HEAD", &uuid);
    let Ok(_) = repo.checkout_head(Some(CheckoutBuilder::new().force())) else {
        log::error!(target: "remote_text_server::get_file_contents", "[{}] Unable to checkout", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
                        log::error!(target: "remote_text_server::get_file_contents", "[{}] Cannot convert filename '{:?}' to string", &uuid, fname.clone());
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    };
                    log::info!(target: "remote_text_server::get_file_contents", "[{}] Found file {}", &uuid, filename);
                    return Ok((filename, content.to_string()));
                } else {
                    log::error!(target: "remote_text_server::get_file_contents", "[{}] No file found in repo", &uuid);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            } else {
                log::error!(target: "remote_text_server::get_file_contents", "[{}] Cannot read repo dir", &uuid);
                eprintln!("Cannot read repo dir for UUID {}", &uuid);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        } else {
            log::error!(target: "remote_text_server::get_file_contents", "[{}] Parent to git dir does not exist", &uuid);
            eprintln!("Parent to git dir does not exist for UUID {}", &uuid);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    } else {
        log::error!(target: "remote_text_server::get_file_contents", "[{}] No repo exists", &uuid);
        eprintln!("No repo exists for UUID {}", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
    match fs::remove_dir_all(FILES_DIR().join(uuid_string)) {
        Ok(_) => {
            log::info!(target: "remote_text_server::delete_file", "[{}] Target directory successfully removed", &obj.id);
            return Ok(Box::new(StatusCode::OK))
        },
        Err(_) => {
            log::error!(target: "remote_text_server::delete_file", "[{}] Target directory was unable to be removed", &obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/*
// PREVIEW FILE //
pdflatex -output-directory {} {}, this_commit_path, input_file_path

TODO: Comment preview_file() functionality & general description
TODO: do

*/
pub(crate) async fn preview_file(obj: FileIDAndGitHash, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    log::trace!(target: "remote_text_server::preview_file", "[{}] Acquiring lock on hash map", &obj.id);
    let repos = repos.lock().unwrap();
    log::trace!(target: "remote_text_server::preview_file", "[{}] Calling get_file_contents", &obj.id);
    let (filename, content) = match get_file_contents(&obj.id, &obj.hash, &repos) {
        Ok((filename, content)) => (filename, content),
        Err(code) => {
            log::trace!(target: "remote_text_server::preview_file", "[{}] Unable to locate file", &obj.id);
            return Ok(Box::new(code));
        }
    };
    // let Ok((filename, content)) = get_file_contents(obj.id, obj.hash, repos) else {
    //     log::trace!(target: "remote_text_server::preview_file", "[{}] Unable to locate file", &obj.id);
    //     return Ok(Box::new(code));
    // };
    log::trace!(target: "remote_text_server::preview_file", "[{}] Located filename and content", &obj.id);

    let mut parts = filename.rsplit(".");
    let ext = parts.next().unwrap();
    let mut rest = parts.collect::<Vec<_>>();
    if rest.len() == 0 {
        log::warn!(target: "remote_text_server::preview_file", "[{}] No file extension (filename: {})", &obj.id, filename);
        return Ok(Box::new(StatusCode::IM_A_TEAPOT))
    };
    rest.reverse();
    let name_root = rest.join(".");
    let log_name = format!("{name_root}.log");
    log::trace!(target: "remote_text_server::preview_file", "[{}] Log name: {log_name}", &obj.id);
    let status_name = format!("{name_root}.status");
    log::trace!(target: "remote_text_server::preview_file", "[{}] Status name: {status_name}", &obj.id);

    let previews_path = PREVIEWS_DIR().join(&obj.id.to_string());
    log::trace!(target: "remote_text_server::preview_file", "[{}] Creating preview path for file (if it doesn't exist)", &obj.id);
    let Ok(_) = fs::create_dir_all(&previews_path) else {
        log::error!(target: "remote_text_server::preview_file", "[{}] Cannot create preview path ({:?})", &obj.id, previews_path);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
    };

    let this_commit_path = previews_path.join(&obj.hash);
    let log_path = this_commit_path.join(&log_name);
    let status_path = this_commit_path.join(&status_name);
    if this_commit_path.exists() {
        log::trace!(target: "remote_text_server::preview_file", "[{}] Preview path already exists for commit {}", &obj.id, obj.hash);

        let Ok(log_contents) = fs::read_to_string(log_path) else {
            log::error!(target: "remote_text_server::preview_file", "[{}] Although preview path exists, log file does not", &obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        };
        log::trace!(target: "remote_text_server::preview_file", "[{}] Loaded preview log", &obj.id);
        let Ok(status_contents) = fs::read_to_string(status_path) else {
            log::error!(target: "remote_text_server::preview_file", "[{}] Although preview path exists, status code file does not", &obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        };
        log::trace!(target: "remote_text_server::preview_file", "[{}] Loaded preview status", &obj.id);
        let status = match status_contents.as_str() {
            "SUCCESS" => CompilationState::SUCCESS,
            "FAILURE" => CompilationState::FAILURE,
            _ => {
                log::error!(target: "remote_text_server::preview_file", "[{}] Although status code file exists, it has invalid contents", &obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };
        log::trace!(target: "remote_text_server::preview_file", "[{}] Converted preview status to enum", &obj.id);
        log::info!(target: "remote_text_server::preview_file", "[{}] Loaded and returned cached preview", &obj.id);
        return Ok(Box::new(warp::reply::json(&CompilationOutput {
            state: status,
            log: log_contents,
        })));
    }
    log::trace!(target: "remote_text_server::preview_file", "[{}] Preview path does not yet exist for commit {}", &obj.id, obj.hash);

    let Ok(_) = fs::create_dir(&this_commit_path) else {
        log::error!(target: "remote_text_server::preview_file", "[{}] Unable to create preview path for commit {}", &obj.id, obj.hash);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
    };
    log::trace!(target: "remote_text_server::preview_file", "[{}] Created preview path", &obj.id);

    log::trace!(target: "remote_text_server::preview_file", "[{}] Compiling based on file extension ({})", &obj.id, ext);
    match ext {
        "tex" => {
            log::trace!(target: "remote_text_server::preview_file", "[{}] Detected TeX file", &obj.id);

            let output_name = format!("{name_root}.pdf");
            log::trace!(target: "remote_text_server::preview_file", "[{}] Output name: {}", &obj.id, output_name);

            let res = Command::new("pdflatex")
                .args(["-output-directory", this_commit_path.canonicalize().unwrap().to_str().unwrap()])
                .args(["-interaction", "nonstopmode"])
                .arg("-halt-on-error")
                .arg(format!("./files/{}/{filename}", &obj.id))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            // .spawn();
            let Ok(res) = res else {
                log::error!(target: "remote_text_server::preview_file", "[{}] Unable to launch pdflatex", &obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            };
            log::trace!(target: "remote_text_server::preview_file", "[{}] Launched pdflatex", &obj.id);

            let Ok(log_content) = fs::read_to_string(log_path) else {
                log::error!(target: "remote_text_server::preview_file", "[{}] Unable to read pdflatex log", &obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            };
            log::trace!(target: "remote_text_server::preview_file", "[{}] Read pdflatex log", &obj.id);

            let Ok(_) = fs::write(status_path, if res.success() { "SUCCESS" } else { "FAILURE" }) else {
                log::error!(target: "remote_text_server::preview_file", "[{}] Unable to write pdflatex status", &obj.id);
                return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
            };
            log::trace!(target: "remote_text_server::preview_file", "[{}] Wrote pdflatex status", &obj.id);

            log::info!(target: "remote_text_server::preview_file", "[{}] Returning status and log of pdflatex preview attempt", &obj.id);
            return Ok(Box::new(warp::reply::json(&CompilationOutput {
                state: if res.success() { CompilationState::SUCCESS } else { CompilationState::FAILURE },
                log: log_content,
            })));
        },
        "md" | "markdown" => {
            log::trace!(target: "remote_text_server::preview_file", "[{}] Detected Markdown file", &obj.id);

            log::trace!(target: "remote_text_server::preview_file", "[{}] Converting with Pandoc", &obj.id);
            match convert_with_pandoc(&obj.id, &name_root, &filename, &this_commit_path, &log_path, &status_path) {
                Ok(output) => {
                    log::trace!(target: "remote_text_server::preview_file", "[{}] Got CompilationOutput", &obj.id);
                    return Ok(Box::new(warp::reply::json(&output)));
                },
                Err(code) => {
                    log::trace!(target: "remote_text_server::preview_file", "[{}] Internal server error", &obj.id);
                    return Ok(Box::new(code));
                }
            }
        }
        _ => {
            log::info!(target: "remote_text_server::preview_file", "[{}] Unknown file type ({})", &obj.id, ext);

            log::trace!(target: "remote_text_server::preview_file", "[{}] Attempting to convert with Pandoc", &obj.id);
            match convert_with_pandoc(&obj.id, &name_root, &filename, &this_commit_path, &log_path, &status_path) {
                Ok(output) => {
                    log::trace!(target: "remote_text_server::preview_file", "[{}] Got CompilationOutput", &obj.id);
                    return Ok(Box::new(warp::reply::json(&output)));
                },
                Err(code) => {
                    log::trace!(target: "remote_text_server::preview_file", "[{}] Internal server error", &obj.id);
                    return Ok(Box::new(code));
                }
            }
        }
    }
}

fn convert_with_pandoc(uuid: &Uuid, name_root: &String, filename: &String, this_commit_path: &PathBuf, log_path: &PathBuf, status_path: &PathBuf) -> Result<CompilationOutput, StatusCode> {
    let output_name = format!("{name_root}.html");
    log::trace!(target: "remote_text_server::preview_file_pandoc", "[{}] Output name: {}", &uuid, output_name);

    let res = Command::new("pandoc")
        .arg("--verbose")
        .arg("-s")
        .args(["-o", this_commit_path.canonicalize().unwrap().join(output_name).to_str().unwrap()])
        .arg(FILES_DIR().join(&uuid.to_string()).join(filename))
        .output();

    let Ok(res) = res else {
        log::error!(target: "remote_text_server::preview_file_pandoc", "[{}] Unable to launch pandoc", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    log::trace!(target: "remote_text_server::preview_file_pandoc", "[{}] Ran pandoc", &uuid);

    let Ok(_) = fs::write(log_path, &res.stderr) else {
        log::error!(target: "remote_text_server::preview_file_pandoc", "[{}] Unable to write pandoc log", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    log::trace!(target: "remote_text_server::preview_file_pandoc", "[{}] Wrote pandoc log to file", &uuid);

    let Ok(log_content) = std::str::from_utf8(&res.stderr) else {
        log::error!(target: "remote_text_server::preview_file_pandoc", "[{}] Pandoc log is not UTF-8", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    log::trace!(target: "remote_text_server::preview_file_pandoc", "[{}] Validated pandoc log as UTF-8", &uuid);

    let Ok(_) = fs::write(status_path, if res.status.success() { "SUCCESS" } else { "FAILURE" }) else {
        log::error!(target: "remote_text_server::preview_file_pandoc", "[{}] Unable to write pandoc status", &uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    log::trace!(target: "remote_text_server::preview_file_pandoc", "[{}] Wrote pandoc status", &uuid);

    log::info!(target: "remote_text_server::preview_file_pandoc", "[{}] Returning status and log of pandoc preview attempt", &uuid);
    return Ok(CompilationOutput {
        state: if res.status.success() { CompilationState::SUCCESS } else { CompilationState::FAILURE },
        log: log_content.to_string(),
    });
}


/// Given a file ID and a commit hash for that file, return an already-previewed file
///
/// ## Arguments
///
/// * `obj`: the file ID and the commit hash, from the JSON data in the HTTP request
/// * `repos`: The lock on the list of files. Currently unused
///
/// ## Returns
///
/// * if the preview exists and can be successfully read, the contents of the previewed file
/// * if the file was never previewed or the preview failed, HTTP 404
/// * HTTP 500 otherwise (primarily when files cannot be read)
pub(crate) async fn get_preview(obj: FileIDAndGitHash, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<Box<dyn warp::Reply>, Infallible> {
    // `preview_path` looks like `PREVIEWS_DIR/f204bae2-4c98-4952-86e6-cb02bc72049b/a0a81fdd89425113d9c1703401039c68ee3d855e`
    let preview_path = PREVIEWS_DIR().join(obj.id.to_string()).join(obj.hash);
    log::trace!(target: "remote_text_server::get_preview", "[{}] Looking for preview path '{:?}'", obj.id, preview_path);
    if !preview_path.exists() {
        // If the path does not exist, then the file was never previewed
        log::info!(target: "remote_text_server::get_preview", "[{}] Preview path does not exist", obj.id);
        return Ok(Box::new(StatusCode::NOT_FOUND))
    }

    let Ok(entries) = fs::read_dir(preview_path) else {
        // I think this error would only happen if the permissions on the folder don't allow us
        //   to read it, since we already know that it exists
        log::error!(target: "remote_text_server::get_preview", "[{}] Cannot read preview path, though it exists", obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
    };

    let items = entries.into_iter() // Iterate over all entries in dir
        .filter_map(|entry| entry.ok())// entry exists (I'm not sure when this would fail)
        .filter_map(|entry| { // Fetch the filetype (e.g., file, dir, symlink) of this entry
            let file_type = entry.file_type();
            Some((entry, file_type.ok()?))
        })
        .filter(|(entry, file_type)| file_type.is_file())// entry is file
        .filter_map(|(entry, _)| { // Fetch the name of this file
            let file_name = entry.file_name().into_string();
            log::trace!(target: "remote_text_server::get_preview", "[{}] Found file {:?}", obj.id, file_name);
            Some((entry, file_name.ok()?))
        })
        .map(|(entry, file_name)| { // Fetch the path of this file
            let path = entry.path();
            (file_name, path)
        })
        .filter_map(|(file_name, path)| { // Fetch the extension of this file
            let ext = path.extension()?.to_owned();
            Some((file_name, path, ext))
        })
        .filter_map(|(file_name, path, ext)| { // Convert from OsString
            Some((file_name, path, ext.to_str()?.to_owned()))
        })
        .collect::<Vec<_>>(); // Collect into Vec (i.e. list/array)

    let Some((status_file_name, status_path, _)) = items.iter().find(|(_, _, extension)| {
        extension == &"status" // First file with a ".status" extension
    }) else {
        // If we can't find it, either the file was never previewed or the status file has disappeared
        // Since we know the file has been previewed, it's unclear what's happened, so bail
        log::error!(target: "remote_text_server::get_preview", "[{}] Cannot locate preview status file", obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    let Ok(status_contents) = fs::read_to_string(status_path) else {
        // Probably a permissions error again
        log::error!(target: "remote_text_server::get_preview", "[{}] Cannot read preview status file {status_file_name}", &obj.id);
        return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
    };
    log::trace!(target: "remote_text_server::get_preview", "[{}] Loaded preview status", &obj.id);
    match status_contents.as_str() {
        "SUCCESS" => {
            // Previewing the file was successful; continue fetching previewed file
            log::trace!(target: "remote_text_server::get_preview", "[{}] Status of previewing was success", obj.id);
        },
        "FAILURE" => {
            // Previewing the file was unsuccessful; why are they trying?
            log::info!(target: "remote_text_server::get_preview", "[{}] Status of previewing was failure", obj.id);
            return Ok(Box::new(StatusCode::NOT_FOUND));
        },
        _ => {
            // Status file is corrupt
            log::error!(target: "remote_text_server::get_preview", "[{}] Although status code file exists, it has invalid contents", &obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    if let Some((file_name, path, _)) = items.iter().find(|(_, _, extension)| {
        extension == &"pdf" // First file with a ".pdf" extension
    }) {
        log::trace!(target: "remote_text_server::get_preview", "[{}] Found PDF file {file_name}", obj.id);
        let Ok(data) = fs::read(path) else {
            // Probably a permissions error
            log::error!(target: "remote_text_server::get_preview", "[{}] Cannot read previewed file", obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        };
        log::trace!(target: "remote_text_server::get_preview", "[{}] Read {} bytes", obj.id, data.len());
        let mut resp = Response::new(Body::from(data));
        // This header is mostly unnecessary, I _think_, because most browsers perform "MIME sniffing"
        //   and detect the contents as a PDF on their own, but this is best practice, potentially
        //   necessary on some clients, and makes it easier to detect the type of output file
        resp.headers_mut().insert("content-type", HeaderValue::from_static("application/pdf"));
        log::info!(target: "remote_text_server::get_preview", "[{}] Returning previewed PDF", obj.id);
        return Ok(Box::new(resp));
    }
    if let Some((file_name, path, _)) = items.iter().find(|(_, _, extension)| {
        extension == &"html" // First file with a ".html" extension
    }) {
        log::trace!(target: "remote_text_server::get_preview", "[{}] Found HTML file {file_name}", obj.id);
        let Ok(data) = fs::read(path) else {
            // Probably a permissions error
            log::error!(target: "remote_text_server::get_preview", "[{}] Cannot read previewed file", obj.id);
            return Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR));
        };
        log::trace!(target: "remote_text_server::get_preview", "[{}] Read {} bytes", obj.id, data.len());
        // Since HTML is how webpages are expressed, there's no "application/html" MIME type
        let resp = Response::new(Body::from(data));
        log::info!(target: "remote_text_server::get_preview", "[{}] Returning previewed PDF", obj.id);
        return Ok(Box::new(resp));
    }
    // The only output formats we currently use are PDF or HTML, so if neither are found, we're
    //   missing our output file.
    log::error!(target: "remote_text_server::get_preview", "[{}] Neither PDF nor HTML files found", obj.id);
    Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
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
