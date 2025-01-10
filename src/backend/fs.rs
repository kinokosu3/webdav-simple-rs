use std::path::{Path, PathBuf};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs;
use uuid::Uuid;
use std::pin::Pin;
use std::future::Future;

use super::{Backend, ResourceInfo, ResourceMetadata};
use crate::error::WebDavError;

#[derive(Clone)]
pub struct FileSystemBackend {
    root: PathBuf,
}

impl FileSystemBackend {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        if path.as_os_str().is_empty() {
            self.root.clone()
        } else {
            self.root.join(path.strip_prefix("/").unwrap_or(path))
        }
    }
    async fn exists(&self, path: &PathBuf) -> Result<bool, WebDavError> {
        Ok(fs::metadata(path).await.is_ok())
    }
}

#[async_trait]
impl Backend for FileSystemBackend {
    

    async fn get_resource(&self, path: &PathBuf) -> Result<ResourceInfo, WebDavError> {
        let full_path = self.resolve_path(path);
        let metadata = fs::metadata(&full_path)
            .await
            .map_err(|_| WebDavError::NotFound(path.clone()))?;

        let resource_metadata = ResourceMetadata {
            path: path.clone(),
            is_dir: metadata.is_dir(),
            len: metadata.len(),
            modified: metadata.modified()?.into(),
            created: metadata.created().ok().map(|t| t.into()),
            etag: format!("\"{:x}\"", Uuid::new_v4()),
        };

        let children = if metadata.is_dir() {
            let mut entries = Vec::new();
            let mut read_dir = fs::read_dir(&full_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let child_path = path.join(entry.file_name());
                let child_metadata = entry.metadata().await?;
                entries.push(ResourceMetadata {
                    path: child_path,
                    is_dir: child_metadata.is_dir(),
                    len: child_metadata.len(),
                    modified: child_metadata.modified()?.into(),
                    created: child_metadata.created().ok().map(|t| t.into()),
                    etag: format!("\"{:x}\"", Uuid::new_v4()),
                });
            }
            Some(entries)
        } else {
            None
        };

        Ok(ResourceInfo {
            metadata: resource_metadata,
            children,
        })
    }

    async fn read_file(&self, path: &PathBuf) -> Result<Bytes, WebDavError> {
        let full_path = self.resolve_path(path);
        let content = fs::read(&full_path)
            .await
            .map_err(|_| WebDavError::NotFound(path.clone()))?;
        Ok(Bytes::from(content))
    }

    async fn write_file(&self, path: &PathBuf, content: Bytes) -> Result<(), WebDavError> {
        let full_path = self.resolve_path(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        println!("full_path: {:?}", self.exists(&full_path).await);
        if self.exists(&full_path).await? {
            return Err(WebDavError::AlreadyExists(path.clone()));
        }
        fs::write(&full_path, content).await?;
        Ok(())
    }

    async fn create_dir(&self, path: &PathBuf) -> Result<(), WebDavError> {
        let full_path = self.resolve_path(path);
        fs::create_dir_all(&full_path).await?;
        Ok(())
    }

    async fn delete(&self, path: &PathBuf) -> Result<(), WebDavError> {
        let full_path = self.resolve_path(path);
        let metadata = fs::metadata(&full_path).await?;
        if metadata.is_dir() {
            fs::remove_dir_all(&full_path).await?;
        } else {
            fs::remove_file(&full_path).await?;
        }
        Ok(())
    }

    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError> {
        let src_path = self.resolve_path(from);
        let dst_path = self.resolve_path(to);
        
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let metadata = fs::metadata(&src_path).await?;
        if metadata.is_dir() {
            copy_dir_all(&src_path, &dst_path).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
        Ok(())
    }

    async fn move_resource(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError> {
        let src_path = self.resolve_path(from);
        let dst_path = self.resolve_path(to);
        
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::rename(&src_path, &dst_path).await?;
        Ok(())
    }
}

fn copy_dir_all<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(&dst).await?;
        let mut read_dir = fs::read_dir(src).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let ty = entry.file_type().await?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&src_path, &dst_path).await?;
            } else {
                fs::copy(&src_path, &dst_path).await?;
            }
        }
        Ok(())
    })
} 