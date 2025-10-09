// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{Parser, ValueEnum};
use serde::Deserialize;

#[derive(Parser, Deserialize, Debug, Clone, PartialEq, Eq, ValueEnum, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Deserialize, Debug, Clone, Default, Eq, PartialEq, strum::Display, ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum Logs {
    Off,
    #[default]
    Stdout,
    File,
}

#[derive(Parser, Deserialize, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable logs
    #[arg(short, long)]
    #[arg(default_value_t = Logs::Stdout)]
    pub logs: Logs,

    /// Set the log level
    #[arg(short = 'L', long)]
    pub log_level: Option<LogLevel>,
}
