use git2::Repository;
use uuid::Uuid;

use crate::api::{PreviewDetail, PreviewDetailType};
use crate::PREVIEWS_DIR;

pub(crate) async fn get_preview(id: Uuid, hash: String) -> PreviewDetail {
    let preview_path = PREVIEWS_DIR.join(id.to_string()).join(hash);
    let exists = preview_path.exists();
    return PreviewDetail {
        name: "".to_string(),
        id,
        r#type: PreviewDetailType::PDF,
        data: "".to_string(),
    }
}