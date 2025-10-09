// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::anyhow;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing::{error, info, instrument};

use crate::{APP_DIR, CONFIG_FILE, XDG_CONFIG_HOME, util};

const DEFAULT_TICK_RATE: u32 = 2;
const DEFAULT_LOW_LEVEL: u32 = 20;
const DEFAULT_CRITICAL_LEVEL: u32 = 5;
const DEFAULT_FULL_DESIGN: bool = true;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Notification {
    pub summary: String,
    pub body: Option<String>,
    pub icon: Option<String>,
    pub urgency: Option<Urgency>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserConfig {
    pub tick_rate: Option<u32>,
    pub bat_name: Option<String>,
    pub low_level: Option<u32>,
    pub critical_level: Option<u32>,
    pub full_design: Option<bool>,
    pub critical: Option<Notification>,
    pub low: Option<Notification>,
    pub full: Option<Notification>,
    pub charging: Option<Notification>,
    pub discharging: Option<Notification>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub tick_rate: u32,
    pub bat_name: Option<String>,
    pub low_level: u32,
    pub critical_level: u32,
    pub full_design: bool,
    pub critical: Option<Notification>,
    pub low: Option<Notification>,
    pub full: Option<Notification>,
    pub charging: Option<Notification>,
    pub discharging: Option<Notification>,
}

impl Config {
    #[instrument]
    pub fn new() -> anyhow::Result<Self> {
        let home = env::var("HOME")?;
        let mut config_dir = env::var(XDG_CONFIG_HOME)
            .map(PathBuf::from)
            .unwrap_or_else(|_| Path::new(&home).join(".config"));
        config_dir.push(APP_DIR);
        util::check_dir_or_create(&config_dir)?;

        let config_file = config_dir.join(CONFIG_FILE);
        info!("config file: {:?}", config_file);
        let content = fs::read_to_string(&config_file).map_err(|e| {
            let error = format!(
                "failed to read config file {}: {}",
                config_file.display(),
                e
            );
            error!(error);
            anyhow!(error)
        })?;
        let config: UserConfig = toml::from_str(&content).map_err(|e| {
            let error = format!(
                "failed to parse config file {}: {}",
                config_file.display(),
                e
            );
            error!(error);
            anyhow!(error)
        })?;
        Ok(config.into())
    }
}

impl From<UserConfig> for Config {
    fn from(config: UserConfig) -> Self {
        Config {
            tick_rate: config.tick_rate.unwrap_or(DEFAULT_TICK_RATE),
            bat_name: config.bat_name,
            low_level: config.low_level.unwrap_or(DEFAULT_LOW_LEVEL),
            critical_level: config.critical_level.unwrap_or(DEFAULT_CRITICAL_LEVEL),
            full_design: config.full_design.unwrap_or(DEFAULT_FULL_DESIGN),
            critical: config.critical,
            low: config.low,
            full: config.full,
            charging: config.charging,
            discharging: config.discharging,
        }
    }
}

impl From<&Urgency> for notify_rust::Urgency {
    fn from(value: &Urgency) -> Self {
        match value {
            Urgency::Low => notify_rust::Urgency::Low,
            Urgency::Normal => notify_rust::Urgency::Normal,
            Urgency::Critical => notify_rust::Urgency::Critical,
        }
    }
}
