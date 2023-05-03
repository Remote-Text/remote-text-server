use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod normal_date_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%SZ";
    // 2023-04-14T06:29:29Z

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

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
    #[serde(with = "normal_date_format")]
    pub(crate) edited_time: DateTime<Utc>,
    #[serde(with = "normal_date_format")]
    pub(crate) created_time: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CreateFileResult {
    pub(crate) name: String,
    pub(crate) id: Uuid,
    pub(crate) hash: String,
    #[serde(with = "normal_date_format")]
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