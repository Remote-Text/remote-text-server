use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{Repository, Sort};
use uuid::Uuid;

use crate::api::FileSummary;

pub(crate) fn repos() -> Arc<Mutex<HashMap<Uuid, Repository>>> {
    Arc::new(Mutex::new(if let Ok(entries) = fs::read_dir("./files/") {
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

            log::trace!(target: "remote_text_server::list_files", "[{}] Revwalking", uuid);
            let mut x = repo.revwalk().ok().unwrap();
            _ = x.set_sorting(Sort::TIME);
            _ = x.push_head();
            let oid = x.next().unwrap().unwrap();
            log::trace!(target: "remote_text_server::list_files", "[{}] Found most recent commit ({})", uuid, oid.to_string());
            let c = repo.find_commit(oid).unwrap();
            let _d = NaiveDateTime::from_timestamp_opt(c.time().seconds(), 0).unwrap();
            let d: DateTime<Utc> = DateTime::from_utc(_d, Utc);
            log::trace!(target: "remote_text_server::list_files", "[{}] Found most recent timestamp ({})", uuid, d.to_string());

            let Some(_oid) = x.last() else {
                log::trace!(target: "remote_text_server::list_files", "[{}] First commit is last commit ({})", uuid, oid.to_string());
                return FileSummary {
                    name: filename,
                    id: *uuid,
                    edited_time: d,
                    created_time: d,
                }
            };
            let Some(oid) = _oid.ok() else {
                log::trace!(target: "remote_text_server::list_files", "[{}] Oldest commit is invalid", uuid);
                return FileSummary {
                    name: filename,
                    id: *uuid,
                    edited_time: d,
                    created_time: d,
                }
            };
            log::trace!(target: "remote_text_server::list_files", "[{}] Found oldest commit ({})", uuid, oid.to_string());
            let c = repo.find_commit(oid).unwrap();
            let _d = NaiveDateTime::from_timestamp_opt(c.time().seconds(), 0).unwrap();
            let d2: DateTime<Utc> = DateTime::from_utc(_d, Utc);
            log::trace!(target: "remote_text_server::list_files", "[{}] Found oldest timestamp ({})", uuid, d2.to_string());
            //git log --all -1 --format=%cd
            FileSummary {
                name: filename,
                id: *uuid,
                edited_time: d,
                created_time: d2,
            }
        }).collect::<Vec<FileSummary>>();
    log::info!(target: "remote_text_server::list_files", "Found {} file(s)", list.len());
    return list;
}