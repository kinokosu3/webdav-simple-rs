use crate::backend::{Backend, ResourceInfo, ResourceMetadata, WebDavError};
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::io::AsyncRead;

use super::handle::list;

pub struct QuarkBackend {
    path_map: DashMap<String, ResourceMetadata>,
}

impl QuarkBackend {
    pub fn new() -> Self {
        Self {
            path_map: DashMap::new(),
        }
    }
}

#[async_trait]
impl Backend for QuarkBackend {
    async fn get_resource(&self, path: &PathBuf) -> Result<ResourceInfo, WebDavError> {
        list(&self.path_map, path.to_str().unwrap()).await
    }

    async fn read_file(&self, path: &PathBuf) -> Result<Bytes, WebDavError> {
        todo!()
    }

    async fn write_file(
        &self,
        path: &PathBuf,
        content: Pin<Box<dyn AsyncRead + Send>>,
    ) -> Result<(), WebDavError> {
        todo!()
    }

    async fn create_dir(&self, path: &PathBuf) -> Result<(), WebDavError> {
        todo!()
    }

    async fn delete(&self, path: &PathBuf) -> Result<(), WebDavError> {
        todo!()
    }

    async fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError> {
        todo!()
    }

    async fn move_resource(&self, from: &PathBuf, to: &PathBuf) -> Result<(), WebDavError> {
        todo!()
    }
}
