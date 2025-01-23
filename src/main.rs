use axum::{
    routing::any,
    Router,
    extract::Path,
    body::Body,
    http::{Request, Method, Response, StatusCode},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::{Level, info, error};
use webdav_rs::{
    backend::{fs::driver::FileSystemBackend, quark::driver::QuarkBackend, Backend}, error::WebDavError, handler::WebDavHandler
};

mod config;
mod logger;

use config::Config;

#[tokio::main]
async fn main() {
    // 获取全局配置
    let config = Config::get();
    
    // 初始化日志
    logger::init(config);

    // 打印配置信息
    info!("配置初始化成功: {:?}", config);

    // 创建后端
    let backend: Arc<dyn Backend> = if config.storage.backend == "quark" {
        Arc::new(QuarkBackend::new())
    } else {
        Arc::new(FileSystemBackend::new(&config.storage.filesystem.root_path))
    };
    let handler = WebDavHandler::new(backend);

    // 创建路由
    let app = Router::new()
        .route(
            "/*path",
            any(move |method: Method, path: Path<String>, req: Request<Body>| {
                let handler = handler.clone();
                let prefix = config.server.prefix.clone();
            
                async move {
                    let path_str = path.0.clone();
                    info!(
                        method = %method,
                        path = %path_str,
                        headers = ?req.headers(),
                        "Handling WebDAV request"
                    );
                    
                let result = if path_str.starts_with(&format!("{}/", prefix)) {
                    // 去掉前缀
                    let origin_path = Path(path_str.replacen(&format!("{}/", prefix), "", 1));
                    match method.as_str() {
                        "PROPFIND" => handler.handle_propfind(origin_path, req).await,
                        "GET" => handler.handle_get(origin_path).await,
                        "PUT" => handler.handle_put(origin_path, req).await,
                        "MKCOL" => handler.handle_mkcol(origin_path).await,
                        "DELETE" => handler.handle_delete(origin_path).await,
                        "COPY" => handler.handle_copy(origin_path, req).await,
                        "MOVE" => handler.handle_move(origin_path, req).await,
                        "OPTIONS" => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("Allow", "OPTIONS, GET, HEAD, POST, PUT, DELETE, PROPFIND, MKCOL, COPY, MOVE")
                            .header("DAV", "1, 2")
                            .body(Body::empty())
                            .unwrap()),
                        _ => Err(WebDavError::InvalidInput("Method not allowed".to_string())),
                    }
                } else {
                    Err(WebDavError::InvalidInput(format!("Path must start with {}/", prefix)))
                };
                    
                    match &result {
                        Ok(response) => info!(
                            method = %method,
                            path = %path_str,
                            status = ?response.status(),
                            "Request completed successfully"
                        ),
                        Err(e) => error!(
                            method = %method,
                            path = %path_str,
                            error = %e,
                            "Request failed"
                        ),
                    }

                    result
                }
            }),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
        );

    // 启动服务器
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>().expect("Invalid host address"),
        config.server.port
    ));
    info!("WebDAV server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
