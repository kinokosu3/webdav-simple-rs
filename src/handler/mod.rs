use axum::{
    body::{Body, Bytes},
    extract::Path,
    http::{header, Request, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use std::sync::Arc;
use tokio_util::io::StreamReader;

use crate::{backend::Backend, error::WebDavError, xml};

const DESTINATION: &str = "destination";

#[derive(Clone)]
pub struct WebDavHandler {
    backend: Arc<dyn Backend>,
}

impl WebDavHandler {
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self { backend }
    }

    pub async fn handle_propfind(
        &self,
        path: Path<String>,
        _req: Request<Body>,
    ) -> Result<Response<Body>, WebDavError> {
        let path = std::path::PathBuf::from(path.0);
        let resource = self.backend.get_resource(&path).await?;

        let mut resources = vec![resource.metadata];
        if let Some(children) = resource.children {
            resources.extend(children);
        }

        let xml_response = xml::create_multistatus_response(&resources)
            .map_err(|e| WebDavError::Internal(e.to_string()))?;

        Ok(Response::builder()
            .status(StatusCode::MULTI_STATUS)
            .header(header::CONTENT_TYPE, "application/xml")
            .body(Body::from(xml_response))
            .unwrap())
    }

    pub async fn handle_get(&self, path: Path<String>) -> Result<Response<Body>, WebDavError> {
        let path = std::path::PathBuf::from(path.0);
        let resource = self.backend.get_resource(&path).await?;

        if resource.metadata.is_dir {
            return Err(WebDavError::InvalidInput(
                "Cannot GET a directory".to_string(),
            ));
        }

        let content = self.backend.read_file(&path).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::ETAG, &resource.metadata.etag)
            .header(
                header::LAST_MODIFIED,
                resource.metadata.modified.to_rfc2822(),
            )
            .header(header::CONTENT_LENGTH, resource.metadata.len)
            .body(Body::from(content))
            .unwrap())
    }

    pub async fn handle_put(
        &self,
        path: Path<String>,
        req: Request<Body>,
    ) -> Result<Response<Body>, WebDavError> {
        let path = std::path::PathBuf::from(path.0);

        let body = req.into_body();
        let stream =
            Box::pin(StreamReader::new(body.into_data_stream().map(|r| {
                r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })));

        self.backend.write_file(&path, stream).await?;

        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn handle_mkcol(&self, path: Path<String>) -> Result<Response<Body>, WebDavError> {
        let path = std::path::PathBuf::from(path.0);
        // 判断是否存在应该交给实现判断
        self.backend.create_dir(&path).await?;

        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn handle_delete(&self, path: Path<String>) -> Result<Response<Body>, WebDavError> {
        let path = std::path::PathBuf::from(path.0);

        self.backend.delete(&path).await?;

        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn handle_copy(
        &self,
        path: Path<String>,
        req: Request<Body>,
    ) -> Result<Response<Body>, WebDavError> {
        let from = std::path::PathBuf::from(path.0);
        let destination = req
            .headers()
            .get(DESTINATION)
            .ok_or_else(|| WebDavError::InvalidInput("Destination header required".to_string()))?
            .to_str()
            .map_err(|_| WebDavError::InvalidInput("Invalid destination header".to_string()))?;

        let to = std::path::PathBuf::from(destination);

        self.backend.copy(&from, &to).await?;

        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn handle_move(
        &self,
        path: Path<String>,
        req: Request<Body>,
    ) -> Result<Response<Body>, WebDavError> {
        let from = std::path::PathBuf::from(path.0);
        let destination = req
            .headers()
            .get(DESTINATION)
            .ok_or_else(|| WebDavError::InvalidInput("Destination header required".to_string()))?
            .to_str()
            .map_err(|_| WebDavError::InvalidInput("Invalid destination header".to_string()))?;

        let to = std::path::PathBuf::from(destination);

        self.backend.move_resource(&from, &to).await?;

        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap())
    }
}

impl IntoResponse for WebDavError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status_code())
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}
