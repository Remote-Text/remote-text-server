use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{IndexAddOption, Repository, Signature, Sort, Time};
use git2::build::CheckoutBuilder;
use uuid::Uuid;

use crate::api::{CreateFileResult, FileSummary};
use crate::FILES_DIR;

pub(crate) fn repos() -> Arc<Mutex<HashMap<Uuid, Repository>>> {
    Arc::new(Mutex::new(if let Ok(entries) = fs::read_dir(FILES_DIR()) {
        entries.into_iter()
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok();
                Some((entry, file_type?))
            })
            .filter(|(_, file_type)| file_type.is_dir())
            .filter_map(|(entry, _)| {
                let uuid = Uuid::parse_str(entry.file_name().to_str()?).ok();
                Some((entry, uuid?))
            })
            .filter_map(|(entry, uuid)| Some((uuid, Repository::open(entry.path()).ok()?)))
            .map(|(uuid, repo)| {
                log::info!(target: "remote_text_server::repositories", "Detected {}", uuid);
                (uuid, repo)
            })
            .collect()
    } else {
        HashMap::new()
    }))
}

pub(crate) fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Vec<FileSummary> {
    log::trace!(target: "remote_text_server::list_files", "Listing files");
    let list = repos.lock().unwrap().iter()
        .map(|(uuid, repo)| {
            if !Path::new(repo.path()).exists() {
                log::error!(target: "remote_text_server::list_files", "[{}] Repository does not exist", uuid);
                panic!()
            }
            let Some(path) = repo.path().parent() else {
                log::error!(target: "remote_text_server::list_files", "[{}] Parent of .git dir does not exist", uuid);
                panic!()
            };
            let Ok(entries) = fs::read_dir(path) else {
                log::error!(target: "remote_text_server::list_files", "[{}] Cannot read entries in directory", uuid);
                panic!()
            };

            log::trace!(target: "remote_text_server::list_files", "[{}] Creating revwalker", uuid);
            let mut walker = repo.revwalk().ok().unwrap();
            _ = walker.set_sorting(Sort::TIME);
            _ = walker.push_head();
            _ = repo.branches(None).map(|branches| {
                branches
                    .filter_map(|branch| branch.ok())
                    // .for_each(|(branch, _)| {
                    //     let Ok(Some(name)) = branch.name() else {
                    //         return
                    //     };
                    //     println!("pushing {name}");
                    //     // let y = x.push_ref(name);
                    //     let y = x.push_ref(format!("refs/heads/{name}").as_str());
                    //     println!("{:?}", y);
                    // })
                    // .filter_map(|(branch, _)| branch.get().target())
                    // .for_each(|oid| {
                    //     log::trace!(target: "remote_text_server::list_files", "[{}] Pushing ")
                    //     let y = x.push(oid);
                    //     println!("{:?}", y);
                    // })
                    .filter_map(|(branch, _)| {
                        let target = branch.get().target();
                        Some((branch, target?))
                    })
                    .for_each(|(branch, oid)| {
                        log::trace!(target: "remote_text_server::list_files", "[{}] Pushing {:?} to refwalker", uuid, branch.name());
                        let Ok(_) = walker.push(oid) else {
                            log::warn!(target: "remote_text_server::list_files", "[{}] Failed to push {:?}", uuid, branch.name());
                            return
                        };
                    })
                    // .filter_map(|(branch, _)| {
                    //     Some((branch.name().ok().to_owned(), branch.get().target()?))
                    // })
                    // .for_each(|(name, oid)| {
                    //     log::trace!(target: "remote_text_server::list_files", "[{}] Pushing {:?} to refwalker", uuid, name);
                    //     let Ok(_) = x.push(oid) else {
                    //         log::warn!(target: "remote_text_server::list_files", "[{}] Failed to push {:?}", uuid, name);
                    //         return
                    //     };
                    // })
            });
            let first_oid = walker.next().unwrap().unwrap();
            log::trace!(target: "remote_text_server::list_files", "[{}] Found most recent commit; setting HEAD ({})", uuid, first_oid.to_string());

            let Ok(_) = repo.set_head_detached(first_oid) else {
                //Not sure why we'd get this error if we know that the commit exists
                ////The hash we were given does not exist
                log::error!(target: "remote_text_server::list_files", "[{}] Unable to set HEAD", &uuid);
                panic!()
            };
            log::trace!(target: "remote_text_server::list_files", "[{}] Set HEAD; checking out", &uuid);
            let Ok(_) = repo.checkout_head(Some(CheckoutBuilder::new().force())) else {
                log::error!(target: "remote_text_server::list_files", "[{}] Unable to checkout", &uuid);
                panic!()
            };

            let files = entries.into_iter()
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let file_type = entry.file_type().ok();
                    Some((entry, file_type?))
                })
                // .filter_map(|entry| Some((entry, entry.file_type().ok()?)))
                .filter(|(_, file_type)| file_type.is_file())
                .map(|(entry, _)| {
                    entry.file_name()
                })
                .collect::<Vec<_>>();
            if files.len() > 1 {
                log::warn!(target: "remote_text_server::list_files", "[{}] Multiple files found", uuid);
            }
            let Some(fname )= files.first() else {
                log::error!(target: "remote_text_server::list_files", "[{}] No files found", uuid);
                panic!("no files found!")
            };
            let Some(_filename) = fname.to_str() else {
                log::error!(target: "remote_text_server::list_files", "[{}] Cannot convert filename from OsStr to str", uuid);
                panic!("Cannot convert filename {:?}", fname)
            };
            let filename = _filename.to_string();
            log::trace!(target: "remote_text_server::list_files", "[{}] Found filename ({filename})", uuid);

            let first_commit = repo.find_commit(first_oid).unwrap();
            let first_naive_date = NaiveDateTime::from_timestamp_opt(first_commit.time().seconds(), 0).unwrap();
            let first_date: DateTime<Utc> = DateTime::from_utc(first_naive_date, Utc);
            log::trace!(target: "remote_text_server::list_files", "[{}] Found most recent timestamp ({})", uuid, first_date.to_string());

            let Some(_last_oid) = walker.last() else {
                log::trace!(target: "remote_text_server::list_files", "[{}] First commit is last commit", uuid);
                return FileSummary {
                    name: filename,
                    id: *uuid,
                    edited_time: first_date,
                    created_time: first_date,
                }
            };
            let Some(last_oid) = _last_oid.ok() else {
                log::trace!(target: "remote_text_server::list_files", "[{}] Oldest commit is invalid", uuid);
                return FileSummary {
                    name: filename,
                    id: *uuid,
                    edited_time: first_date,
                    created_time: first_date,
                }
            };
            log::trace!(target: "remote_text_server::list_files", "[{}] Found oldest commit ({})", uuid, last_oid.to_string());
            let last_commit = repo.find_commit(last_oid).unwrap();
            let last_naive_date = NaiveDateTime::from_timestamp_opt(last_commit.time().seconds(), 0).unwrap();
            let last_date: DateTime<Utc> = DateTime::from_utc(last_naive_date, Utc);
            log::trace!(target: "remote_text_server::list_files", "[{}] Found oldest timestamp ({})", uuid, last_date.to_string());
            //git log --all -1 --format=%cd
            FileSummary {
                name: filename,
                id: *uuid,
                edited_time: first_date,
                created_time: last_date,
            }
        }).collect::<Vec<FileSummary>>();
    log::info!(target: "remote_text_server::list_files", "Found {} file(s)", list.len());
    return list;
}

pub(crate) fn create_file(file_name: String, file_content: Option<String>, addr: Option<SocketAddr>, repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Result<CreateFileResult, &'static str> {
    let now = Utc::now();
    let uuid = Uuid::new_v4();
    log::info!(target: "remote_text_server::create_file", "[{}] Creating new file", uuid);
    let Ok(repo) = Repository::init(FILES_DIR().join(uuid.to_string())) else {
        log::error!(target: "remote_text_server::create_file", "[{}] Cannot create repository", uuid);
        return Err("Cannot create repository");
    };
    let time = Time::new(now.timestamp(), 0);
    let them = match addr {
        Some(addr) => addr.to_string(),
        None => {
            log::warn!(target: "remote_text_server::create_file", "[{}] Non-socket connection", uuid);
            "Non Socket Remote User".to_string()
        }
    };
    let fp = FILES_DIR().join(uuid.to_string()).join(&file_name);
    let Ok(mut file) = std::fs::File::create(fp) else {
        log::error!(target: "remote_text_server::create_file", "[{}] Unable to create file", uuid);
        return Err("Unable to create file!");
    };
    if let Some(content) = file_content {
        log::trace!(target: "remote_text_server::create_file", "[{}] Writing initial content to file", uuid);
        file.write_all(content.as_ref()).unwrap();
    }
    let their_sig = Signature::new(&them, "blinky@remote-text.com", &time).unwrap();
    let our_sig = Signature::new("Remote Text", "blinky@remote-text.com", &time).unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(&["."], IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let co = repo.commit(Some("HEAD"), &their_sig, &our_sig, "", &repo.find_tree(tree_id).unwrap(), &vec![]).unwrap();
    log::info!(target: "remote_text_server::create_file", "[{}] Made initial commit ({})", uuid, co.to_string());
    let result = CreateFileResult {
        name: file_name,
        id: uuid,
        hash: co.to_string(),
        created_time: now,
    };
    log::trace!(target: "remote_text_server::create_file", "[{}] Inserting new repo into hash map", uuid);
    repos.lock().unwrap().insert(uuid, repo);
    log::trace!(target: "remote_text_server::create_file", "[{}] Inserted new repo into hash map", uuid);
    return Ok(result);
}