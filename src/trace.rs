// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::Result;
use tracing::{debug, info};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

use crate::cli::{Cli, LogLevel, Logs};
use crate::util;

const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
const LOG_DIR: &str = "bato";
const LOG_FILE: &str = "bato.log";
const LOG_FILE_OLD: &str = "bato.old.log";

fn rotate_log_file(log_dir: PathBuf) -> Result<Option<PathBuf>> {
    let log_file = log_dir.join(LOG_FILE);
    if log_file.is_file() {
        let old_file = log_dir.join(LOG_FILE_OLD);
        fs::rename(&log_file, &old_file).inspect_err(|e| {
            eprintln!(
                "failed to rename log file during log rotation {}: {e}",
                log_file.display()
            )
        })?;
        return Ok(Some(old_file));
    }
    Ok(None)
}

fn get_filter(level: &Option<LogLevel>) -> LevelFilter {
    match level {
        Some(LogLevel::Trace) => LevelFilter::TRACE,
        Some(LogLevel::Debug) => LevelFilter::DEBUG,
        Some(LogLevel::Info) => LevelFilter::INFO,
        Some(LogLevel::Warn) => LevelFilter::WARN,
        Some(LogLevel::Error) => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    }
}

pub fn init(cli: &Cli) -> Result<Option<WorkerGuard>> {
    if cli.logs == Logs::Off {
        return Ok(None);
    };

    let filter = EnvFilter::builder()
        .with_default_directive(get_filter(&cli.log_level).into())
        .from_env()?;

    match cli.logs {
        Logs::Stdout => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .init();
            Ok(None)
        }
        Logs::File => {
            let home = env::var("HOME")?;
            let cache_dir = env::var(XDG_CACHE_HOME)
                .map(PathBuf::from)
                .unwrap_or_else(|_| Path::new(&home).join(".cache"));
            let log_dir = cache_dir.join(LOG_DIR);
            util::check_dir_or_create(&log_dir)?;
            let old_file = rotate_log_file(log_dir.clone()).ok().flatten();

            let appender = rolling::never(log_dir.clone(), LOG_FILE);
            let (writer, guard) = tracing_appender::non_blocking(appender);

            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .with_ansi(false)
                .with_writer(writer)
                .init();

            if let Some(file) = old_file {
                debug!("rotated log file: {}", file.display());
            }
            info!("logging to file: {}", log_dir.join(LOG_FILE).display());
            Ok(Some(guard))
        }
        _ => Ok(None),
    }
}
