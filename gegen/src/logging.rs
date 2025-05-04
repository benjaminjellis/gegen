use std::path::PathBuf;

use dirs::data_local_dir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

const MAX_LOG_FILE: usize = 10;

pub(crate) fn create_file_appender() -> RollingFileAppender {
    let data_local_dir = data_local_dir().expect("Failed to find data local directory");
    let gegen_dir = PathBuf::from("gegen/logs");
    let logs_dir = data_local_dir.join(gegen_dir);

    tracing_appender::rolling::RollingFileAppender::builder()
        .max_log_files(MAX_LOG_FILE)
        .rotation(Rotation::HOURLY)
        .filename_prefix("gegen.log")
        .build(logs_dir)
        .expect("failed to build file appender")
}
