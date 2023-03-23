use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) content: String
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileSummary {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) edited_time: DateTime<Utc>,
    pub(crate) created_time: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PreviewDetail {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) r#type: PreviewDetailType,
    pub(crate) data: String
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum PreviewDetailType {
    PDF,
    HTML
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitCommit {
    pub(crate) hash: String,
    pub(crate) parent: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitRef {
    pub(crate) name: String,
    pub(crate) hash: String
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CompilationOutput {
    pub(crate) state: CompilationState,
    pub(crate) log: String
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum CompilationState {
    SUCCESS,
    FAILURE
}

//***

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileIDAndOptionalGitHash {
    pub(crate) id: Uuid,
    pub(crate) hash: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GitHistory {
    pub(crate) commits: Vec<GitCommit>,
    pub(crate) refs: Vec<GitRef>
}