use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// #[derive . . .] adds serialize/deserialize and clone to each struct/type

// Files contain a name, unique ID, and content
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) content: String
}
// File Summary contains a name, unique ID, initial creation time and most recent edit time
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileSummary {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) edited_time: DateTime<Utc>,
    pub(crate) created_time: DateTime<Utc>
}
// Preview Detail has a name, unique ID, a preview type enum, and data/contents
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PreviewDetail {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) r#type: PreviewDetailType,
    pub(crate) data: String
}
// Defines two formats of preview files: PDF & HTML
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum PreviewDetailType {
    PDF,
    HTML
}

// Git Commit tracks the hash of the new commit, and the parent (last commit)
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitCommit {
    pub(crate) hash: String,
    pub(crate) parent: Option<String>
}

// Git Ref tracks the name and hash of any
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitRef {
    pub(crate) name: String,
    pub(crate) hash: String
}
// Compilation Output stores if a compilation attempt was successful or not, as well as the
// corresponding log
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CompilationOutput {
    pub(crate) state: CompilationState,
    pub(crate) log: String
}

// Enum that tracks whether a compilation was successful or has failed
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum CompilationState {
    SUCCESS,
    FAILURE
}

//***

// Contains the unique ID and corresponding hash for a file
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileIDAndOptionalGitHash {
    pub(crate) id: Uuid,
    pub(crate) hash: Option<String>
}

// Git History contains all commits to the repository, and the name/hash of each commit
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitHistory {
    pub(crate) commits: Vec<GitCommit>,
    pub(crate) refs: Vec<GitRef>
}