use std::fs::File;

use log::error;

pub fn setup_logger(log_path: &str) {
    let log_file = Box::new(File::create(log_path).unwrap_or_else(|_| {
        error!("Failed to open log file: '{log_path}'");
        std::process::exit(1);
    }));

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(log_file))
        .format_timestamp_secs()
        .init();
}