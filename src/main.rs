use axum::{
    routing::any,
    Router,
    extract::Path,
    body::Body,
    http::{Request, Method, Response, StatusCode},
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webdav_rs::{
    backend::fs::FileSystemBackend,
    handler::WebDavHandler,
    error::WebDavError,
};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建后端
    let backend = FileSystemBackend::new("./storage");
    let handler = WebDavHandler::new(backend);

    // 创建路由
    let app = Router::new()
        .route(
            "/*path",
            any(move |method: Method, path: Path<String>, req: Request<Body>| {
                let handler = handler.clone();
                async move {
                    match method.as_str() {
                        "PROPFIND" => handler.handle_propfind(path, req).await,
                        "GET" => handler.handle_get(path).await,
                        "PUT" => handler.handle_put(path, req).await,
                        "MKCOL" => handler.handle_mkcol(path).await,
                        "DELETE" => handler.handle_delete(path).await,
                        "COPY" => handler.handle_copy(path, req).await,
                        "MOVE" => handler.handle_move(path, req).await,
                        "OPTIONS" => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("Allow", "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND, MKCOL, COPY, MOVE")
                            .header("DAV", "1, 2")
                            .body(Body::empty())
                            .unwrap()),
                        _ => Err(WebDavError::InvalidInput("Method not allowed".to_string())),
                    }
                }
            }),
        )
        .layer(TraceLayer::new_for_http());

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("WebDAV server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
