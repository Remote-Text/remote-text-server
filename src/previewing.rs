use git2::Repository;
use uuid::Uuid;
use crate::api::{PreviewDetail, PreviewDetailType};

pub(crate) async fn get_preview(id: Uuid, hash: String) -> PreviewDetail {
    let preview_path_str = format!("previews/{}/{}", id.to_string(), hash);
    let preview_path = std::path::Path::new(&preview_path_str);
    let exists = preview_path.exists();
    return PreviewDetail {
        name: "".to_string(),
        id,
        r#type: PreviewDetailType::PDF,
        data: "".to_string(),
    }
}