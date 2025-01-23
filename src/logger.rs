use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::Config;

pub fn init(config: &'static Config) {
    let env_filter = EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| config.server.log_level.clone()),
    );

    if config.log.file_enabled {
        let log_path = Path::new(&config.log.file_path);
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create log directory");
        }

        let file_appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("webdav")
            .filename_suffix("log")
            .build(log_path.parent().unwrap_or_else(|| Path::new(".")))
            .expect("Failed to create file appender");

        fmt::Subscriber::builder()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_writer(file_appender)
            .init();
    } else {
        fmt::Subscriber::builder()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .init();
    }
}
