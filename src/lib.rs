// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod error;
mod notify;
use error::Error;
use notify::send;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
use std::fs;

const SYS_PATH: &str = "/sys/class/power_supply/";
const LOW_LEVEL: u32 = 30;
const BAT_NAME: &str = "BAT0";

#[derive(Debug, Serialize, Deserialize)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    summary: String,
    body: Option<String>,
    icon: Option<String>,
    urgency: Option<Urgency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bat_name: String,
    pub low_level: u32,
    low: Notification,
    full: Notification,
}

pub struct Bato {
    config: Config,
    low_notified: bool,
    full_notified: bool,
}

impl Bato {
    pub fn with_config(config: Config) -> Self {
        Bato {
            config,
            low_notified: false,
            full_notified: false,
        }
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let energy_full = read_and_parse(&format!(
            "{}{}/energy_full_design",
            SYS_PATH, self.config.bat_name
        ))?;
        let energy_now =
            read_and_parse(&format!("{}{}/energy_now", SYS_PATH, self.config.bat_name))?;
        let status = read_and_trim(&format!("{}{}/status", SYS_PATH, self.config.bat_name))?;
        let capacity = energy_full as u64;
        let energy = energy_now as u64;
        let battery_level = u32::try_from(100_u64 * energy / capacity)?;
        if status == "Discharging" && !self.low_notified && battery_level <= self.config.low_level {
            self.low_notified = true;
            send(&self.config.low);
        }
        if status == "Full" && !self.full_notified {
            self.full_notified = true;
            send(&self.config.full);
        }
        if status == "Charging" && self.low_notified {
            self.low_notified = false;
        }
        if status == "Discharging" && self.full_notified {
            self.low_notified = false;
        }
        println!("{:#?}", battery_level);
        Ok(())
    }
}

pub fn deserialize_config() -> Result<Config, Error> {
    let home = env::var("HOME").map_err(|err| format!("environment variable HOME, {}", err))?;
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));
    let content = fs::read_to_string(format!("{}/bato/bato.yaml", config_path))
        .map_err(|err| format!("while reading the config file, {}", err))?;
    let config: Config = serde_yaml::from_str(&content)
        .map_err(|err| format!("while deserializing the config file, {}", err))?;
    Ok(config)
}

fn read_and_trim(file: &str) -> Result<String, Error> {
    let content = fs::read_to_string(file)
        .map_err(|err| format!("error while reading the file \"{}\": {}", file, err))?;
    Ok(content.trim().to_string())
}

fn read_and_parse(file: &str) -> Result<i32, Error> {
    let content = read_and_trim(file)?;
    let data = content
        .parse::<i32>()
        .map_err(|err| format!("error while parsing the file \"{}\": {}", file, err))?;
    Ok(data)
}
