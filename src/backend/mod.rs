use crate::error::WebDavError;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::pin::Pin;
use tokio::io::AsyncRead;

pub mod fs;
pub mod quark;

#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    pub path: PathBuf,
    pub is_dir: bool,
    pub len: u64,
    pub modified: DateTime<Utc>,
    pub created: Option<DateTime<Utc>>,
    pub etag: String,
}

#[derive(Debug)]
pub struct ResourceInfo {
    pub metadata: ResourceMetadata,
    pub children: Option<Vec<ResourceMetadata>>,
}

#[async_trait]
pub trait Backend: Send + Sync + 'static {
    /// 获取资源信息
    async fn get_resource(&self, path: &PathBuf) -> Result<ResourceInfo, WebDavError>;

    /// 读取文件内容
    async fn read_file(&self, path: &PathBuf) -> Result<Bytes, WebDavError>;

    /// 写入文件内容
    async fn write_file(
        &self,
        path: &PathBuf,
        content: Pin<Box<dyn AsyncRead + Send>>,
    ) -> Result<(), WebDavError>;

    /// 创建目录
    async fn create_dir(&self, path: &PathBuf) -> Result<(), WebDavError>;

    /// 删除资源（文件或目录）
    async fn delete(&self, path: &PathBuf) -> Result<(), WebDavError>;
 
    /// 复制资源
    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError>;

    /// 移动资源
    async fn move_resource(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError>;
}
