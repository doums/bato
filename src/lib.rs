// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod error;
mod notify;
use error::Error;
use notify::{close_libnotilus, init_libnotilus, send, NotifyNotification};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
use std::ffi::CString;
use std::fs;
use std::ptr;

const APP_NAME: &str = "bato";
const SYS_PATH: &str = "/sys/class/power_supply/";
const BAT_NAME: &str = "BAT0";

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(C)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    summary: CString,
    body: Option<CString>,
    icon: Option<CString>,
    urgency: Option<Urgency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub tick_rate: Option<u32>,
    pub bat_name: Option<String>,
    pub low_level: u32,
    pub critical_level: u32,
    critical: Notification,
    low: Notification,
    full: Notification,
    charging: Notification,
    discharging: Notification,
}

pub struct Bato {
    bat_name: String,
    notification: *mut NotifyNotification,
    config: Config,
    critical_notified: bool,
    low_notified: bool,
    full_notified: bool,
    status_notified: bool,
    previous_status: String,
}

impl<'a> Bato {
    pub fn with_config(mut config: Config) -> Self {
        let bat_name = if let Some(v) = &config.bat_name {
            String::from(v)
        } else {
            String::from(BAT_NAME)
        };
        if config.critical.urgency.is_none() {
            config.critical.urgency = Some(Urgency::Critical)
        }
        if config.low.urgency.is_none() {
            config.low.urgency = Some(Urgency::Normal)
        }
        if config.full.urgency.is_none() {
            config.full.urgency = Some(Urgency::Low)
        }
        Bato {
            bat_name,
            config,
            critical_notified: false,
            notification: ptr::null_mut(),
            low_notified: false,
            full_notified: false,
            status_notified: false,
            previous_status: "Unknown".to_string(),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let notification = init_libnotilus(APP_NAME);
        if notification.is_null() {
            return Err(Error::new("libnotilus, fail to init"));
        }
        self.notification = notification;
        Ok(())
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let mut current_notification: Option<&Notification> = None;
        let energy_full =
            read_and_parse(&format!("{}{}/energy_full_design", SYS_PATH, self.bat_name))?;
        let energy_now = read_and_parse(&format!("{}{}/energy_now", SYS_PATH, self.bat_name))?;
        let status = read_and_trim(&format!("{}{}/status", SYS_PATH, self.bat_name))?;
        let capacity = energy_full as u64;
        let energy = energy_now as u64;
        let battery_level = u32::try_from(100_u64 * energy / capacity)?;
        if status == "Charging" && self.previous_status != "Charging" && !self.status_notified {
            self.status_notified = true;
            current_notification = Some(&self.config.charging);
        }
        if status == "Discharging" && self.previous_status != "Discharging" && !self.status_notified
        {
            self.status_notified = true;
            current_notification = Some(&self.config.discharging);
        }
        if status == self.previous_status && self.status_notified {
            self.status_notified = false;
        }
        if status == "Discharging"
            && !self.low_notified
            && battery_level <= self.config.low_level
            && battery_level > self.config.critical_level
        {
            self.low_notified = true;
            current_notification = Some(&self.config.low);
        }
        if status == "Discharging"
            && !self.critical_notified
            && battery_level <= self.config.critical_level
        {
            self.critical_notified = true;
            current_notification = Some(&self.config.critical);
        }
        if status == "Full" && !self.full_notified {
            self.full_notified = true;
            current_notification = Some(&self.config.full);
        }
        if status == "Charging" && self.critical_notified {
            self.critical_notified = false;
        }
        if status == "Charging" && self.low_notified {
            self.low_notified = false;
        }
        if status == "Discharging" && self.full_notified {
            self.full_notified = false;
        }
        self.previous_status = status;
        if let Some(notification) = current_notification {
            send(self.notification, notification);
        }
        Ok(())
    }

    pub fn close(&mut self) {
        close_libnotilus(self.notification)
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
