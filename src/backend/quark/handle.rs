use std::path::PathBuf;

use crate::{
    backend::{ResourceInfo, ResourceMetadata},
    error::WebDavError,
};
use chrono::Utc;
use dashmap::DashMap;

use super::{base_client::base_request, types::SortResp};

pub async fn list(
    dm: &DashMap<String, ResourceMetadata>,
    path: &str,
) -> Result<ResourceInfo, WebDavError> {
    // 需要用内存缓存来进行路径和fid的映射
    let root_path = PathBuf::from(path);
    let resource_metadata = if path == "" {
        ResourceMetadata {
            path: root_path.clone(),
            is_dir: true,
            len: 0,
            modified: Utc::now(),
            created: None,
            etag: "0".to_string(),
        }
    } else {
        dm.get(path)
            .ok_or_else(|| WebDavError::NotFound(path.into()))?
            .clone()
    };

    let mut page = 1;
    let size = 100;
    let base_query = vec![
        ("_fetch_total", "1"),
        ("pdir_fid", resource_metadata.etag.as_str()),
        ("_sort", "file_type:asc,updated_at:desc"),
    ];
    let mut children = Vec::new();
    loop {
        let mut query = base_query.clone();
        let page_str = page.to_string();
        let size_str = size.to_string();
        query.push(("_page", page_str.as_str()));
        query.push(("_size", size_str.as_str()));
        let resp: SortResp =
            base_request("/file/sort", reqwest::Method::GET, |req| req.query(&query)).await?;
        for item in resp.data.list {
            children.push(ResourceMetadata {
                path: root_path.join(item.file_name),
                is_dir: !item.file,
                len: item.size as u64,
                modified: chrono::DateTime::from_timestamp(item.updated_at / 1000, 0)
                    .unwrap_or_else(|| Utc::now()),
                created: Some(
                    chrono::DateTime::from_timestamp(item.created_at / 1000, 0)
                        .unwrap_or_else(|| Utc::now()),
                ),
                etag: item.fid,
            })
        }
        if page * size >= resp.metadata.total {
            break;
        }
        page += 1;
    }
    Ok(ResourceInfo {
        metadata: resource_metadata,
        children: Some(children),
    })
}
