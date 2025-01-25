use std::{collections::HashMap, io::Read, path::{Path, PathBuf}, sync::Arc};

use crate::{
    backend::{ResourceInfo, ResourceMetadata}, config::{self, Config}, error::WebDavError
};
use bytes::Bytes;
use chrono::Utc;
use dashmap::DashMap;
use http::header::RANGE;
use reqwest::header::HeaderMap;
use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};
use thiserror::Error;

use super::{base_client::{self, base_request, API_URL}, driver::QuarkBackend, types::SortResp, types::DownResp};

impl From<DownloadError> for WebDavError {
    fn from(error: DownloadError) -> Self {
        WebDavError::Internal(error.to_string())
    }
}

impl QuarkBackend {
    pub async fn list(
        &self,
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
            self.path_map.get(path)
                .ok_or_else(|| WebDavError::NotFound(path.into()))?
                .clone()
        };
        if !resource_metadata.is_dir {
            return Ok(ResourceInfo {
                metadata: resource_metadata,
                children: None,
            });
        }
    
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
                let path = root_path.join(item.file_name);
                let rm = ResourceMetadata {
                    path: path.clone(),
                    is_dir: !item.file,
                    len: item.size as u64,
                    modified: chrono::DateTime::from_timestamp(item.updated_at / 1000, 0)
                        .unwrap_or_else(|| Utc::now()),
                    created: Some(
                        chrono::DateTime::from_timestamp(item.created_at / 1000, 0)
                            .unwrap_or_else(|| Utc::now()),
                    ),
                    etag: item.fid,
                };
                children.push(rm.clone());
                let mut path_str = path.to_str().unwrap().to_string();
                if !item.file {
                    path_str.push('/');
                }
                self.path_map.insert(path_str, rm);
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
    pub async fn get_file(&self, path: &str) -> Result<Bytes, WebDavError> {
        let resource_metadata = self.path_map.get(path)
            .ok_or_else(|| WebDavError::NotFound(path.into()))?
            .clone();
        // 检查临时文件夹中是否有临时缓存，有的直接返回
        let config = Config::get();
        let temp_path = PathBuf::from(&config.storage.temp_path);
        let file_path = temp_path.join(&resource_metadata.etag);
        if file_path.exists() {
            tracing::debug!("命中临时文件夹缓存: {}", file_path.to_str().unwrap());
            let mut file = File::open(file_path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;
            return Ok(Bytes::from(data));
        }
        let mut data = HashMap::new();
        data.insert("fids", vec![resource_metadata.etag.clone()]);
        let resp: DownResp = base_request("/file/download", reqwest::Method::POST, |req| {
            req.json(&data)
        })
        .await?;
        // 在外部定义 headers
        let mut base_headers = HeaderMap::new();
        base_headers.insert("Cookie", config.storage.quark.cookie.parse().map_err(|e| WebDavError::InvalidInput(format!("无效的 Cookie: {}", e)))?);
        base_headers.insert("Accept", "application/json, text/plain, */*".parse().unwrap());
        base_headers.insert("Referer", "https://pan.quark.cn/".parse().unwrap());
        base_headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(base_headers)
            .build()
            .map_err(|e| WebDavError::InvalidInput(format!("无法创建 HTTP 客户端: {}", e)))?;
        let part_size = 256 * 1024; // 256kb
        let data = concurrent_download(client, resp.data[0].download_url.clone(), 2, part_size, 
            resp.data[0].size as usize, 
            resource_metadata.etag.clone()).await?;
        Ok(data)
    }
    
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("服务器不支持断点续传，状态码: {0}")]
    RangeNotSupported(reqwest::StatusCode),
    
    #[error("无法获取 Content-Range 头")]
    MissingContentRange,
    
    #[error("收到的数据超出预期大小")]
    DataSizeExceeded,
    
    #[error("无法获取最终结果")]
    ResultLocked,
    
    #[error("HTTP 请求错误: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("任务执行错误: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

// 获取数据同时缓存到磁盘上
async fn concurrent_download(
    client: reqwest::Client,
    url: String,
    concurrency: usize,
    part_size: usize,
    total_size: usize,
    etag: String,
) -> Result<Bytes, DownloadError> {
    // 计算分片
    let chunks = (total_size + part_size - 1) / part_size;
    let result = Arc::new(Mutex::new(vec![0u8; total_size]));

    // 创建下载任务
    let mut tasks = Vec::with_capacity(chunks);

    for i in 0..chunks {
        let start = i * part_size;
        let end = (start + part_size).min(total_size);
        let url = url.clone();
        let client = client.clone();
        let result = Arc::clone(&result);

        let task = tokio::spawn({
            async move {
                let range = format!("bytes={}-{}", start, end - 1);
                let resp = client
                    .get(&url)
                    .header(RANGE, range.clone())
                    .send()
                    .await?;

                // 检查是否支持断点续传
                if resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                    return Err(DownloadError::RangeNotSupported(resp.status()));
                }

                let content_range = resp
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .ok_or(DownloadError::MissingContentRange)?;

                println!("下载区间: {}, Content-Range: {}", range, content_range);

                let data = resp.bytes().await?;
                let length = data.len();

                let mut buffer = result.lock().await;
                if start + length > total_size {
                    return Err(DownloadError::DataSizeExceeded);
                }
                buffer[start..start + length].copy_from_slice(&data);

                Ok::<_, DownloadError>(())
            }
        });

        tasks.push(task);

        // 控制并发数
        if tasks.len() >= concurrency {
            if let Some(task) = tasks.pop() {
                task.await??;  // 使用双问号来处理嵌套的 Result
            }
        }
    }

    // 等待剩余任务完成
    for task in tasks {
        task.await??;
    }

    // 获取数据并保存到文件
    let data = Arc::try_unwrap(result)
        .map_err(|_| DownloadError::ResultLocked)?
        .into_inner();

    let config = Config::get();
    let temp_path = PathBuf::from(&config.storage.temp_path);
    let file_path = temp_path.join(&etag);
    let mut file = File::create(file_path).await?;
    file.write_all(&data).await?;

    Ok(Bytes::from(data))
}

