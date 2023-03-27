use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{Repository, Sort};
use uuid::Uuid;
use crate::api::{FileSummary, PreviewDetail};

use futures::future;

pub(crate) fn repos() -> Arc<Mutex<HashMap<Uuid, Repository>>> {
    Arc::new(Mutex::new(if let Ok(entries) = fs::read_dir(".") {
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
            .collect()
    } else {
        HashMap::new()
    }))
}

pub(crate) fn list_files(repos: Arc<Mutex<HashMap<Uuid, Repository>>>) -> Vec<FileSummary> {
    repos.lock().unwrap().iter()
        .map(|(uuid, repo)| {

            println!("REFWALKING {}", uuid);
            let mut x = repo.revwalk().ok().unwrap();
            _ = x.set_sorting(Sort::TIME);
            _ = x.push_head();
            let oid = x.next().unwrap().unwrap();
            let c = repo.find_commit(oid).unwrap();
            let _d = NaiveDateTime::from_timestamp_opt(c.time().seconds(), 0).unwrap();
            let d: DateTime<Utc> = DateTime::from_utc(_d, Utc);
            println!("MOST RECENT {}", c.time().seconds());

            let Some(_oid) = x.last() else {
                return FileSummary {
                    name: "TEST".to_string(),
                    id: *uuid,
                    edited_time: d,
                    created_time: d,
                }
            };
            let Some(oid) = _oid.ok() else {
                return FileSummary {
                    name: "TEST".to_string(),
                    id: *uuid,
                    edited_time: d,
                    created_time: d,
                }
            };
            let c = repo.find_commit(oid).unwrap();
            let _d = NaiveDateTime::from_timestamp_opt(c.time().seconds(), 0).unwrap();
            let d2: DateTime<Utc> = DateTime::from_utc(_d, Utc);
            println!("INITIAL {}", c.time().seconds());
            //git log --all -1 --format=%cd
            FileSummary {
                name: "TEST".to_string(),
                id: *uuid,
                edited_time: d,
                created_time: d2,
            }
        }).collect()
}