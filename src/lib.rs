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
const CHARGE_PREFIX: &str = "charge";
const ENERGY_PREFIX: &str = "energy";
const FULL_ATTRIBUTE: &str = "full";
const FULL_DESIGN_ATTRIBUTE: &str = "full_design";
const NOW_ATTRIBUTE: &str = "now";

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
    bat_name: Option<String>,
    low_level: u32,
    critical_level: u32,
    full_design: Option<bool>,
    critical: Option<Notification>,
    low: Option<Notification>,
    full: Option<Notification>,
    charging: Option<Notification>,
    discharging: Option<Notification>,
}

pub struct Bato<'a> {
    bat_name: String,
    attribute_prefix: &'a str,
    notification: *mut NotifyNotification,
    config: Config,
    critical_notified: bool,
    low_notified: bool,
    full_design: bool,
    full_notified: bool,
    status_notified: bool,
    previous_status: String,
}

impl<'a> Bato<'a> {
    pub fn with_config(mut config: Config) -> Result<Self, Error> {
        let bat_name = if let Some(v) = &config.bat_name {
            String::from(v)
        } else {
            String::from(BAT_NAME)
        };
        if let Some(v) = &mut config.critical {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Critical)
            }
        }
        if let Some(v) = &mut config.low {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Normal)
            }
        }
        if let Some(v) = &mut config.full {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Normal)
            }
        }
        let mut full_design = true;
        if let Some(v) = config.full_design {
            full_design = v;
        }
        let attribute_prefix = find_attribute_prefix(&bat_name)?;
        Ok(Bato {
            bat_name,
            attribute_prefix,
            config,
            critical_notified: false,
            notification: ptr::null_mut(),
            full_design,
            low_notified: false,
            full_notified: false,
            status_notified: false,
            previous_status: "Unknown".to_string(),
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        let notification = init_libnotilus(APP_NAME);
        if notification.is_null() {
            return Err(Error::new("libnotilus, fail to init"));
        }
        self.notification = notification;
        Ok(())
    }

    fn attribute(&self, attribute: &str) -> Result<i32, Error> {
        read_and_parse(&format!(
            "{}{}/{}_{}",
            SYS_PATH, self.bat_name, self.attribute_prefix, attribute
        ))
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let mut current_notification: Option<&Notification> = None;
        let capacity = match self.full_design {
            true => self.attribute(FULL_DESIGN_ATTRIBUTE)?,
            false => self.attribute(FULL_ATTRIBUTE)?,
        } as u64;
        let energy = self.attribute(NOW_ATTRIBUTE)? as u64;
        let status = read_and_trim(&format!("{}{}/status", SYS_PATH, self.bat_name))?;
        let battery_level = u32::try_from(100_u64 * energy / capacity)?;
        if status == "Charging" && self.previous_status != "Charging" && !self.status_notified {
            self.status_notified = true;
            if let Some(n) = &self.config.charging {
                current_notification = Some(n);
            }
        }
        if status == "Discharging" && self.previous_status != "Discharging" && !self.status_notified
        {
            self.status_notified = true;
            if let Some(n) = &self.config.discharging {
                current_notification = Some(n);
            }
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
            if let Some(n) = &self.config.low {
                current_notification = Some(n);
            }
        }
        if status == "Discharging"
            && !self.critical_notified
            && battery_level <= self.config.critical_level
        {
            self.critical_notified = true;
            if let Some(n) = &self.config.critical {
                current_notification = Some(n);
            }
        }
        if status == "Full" && !self.full_notified {
            self.full_notified = true;
            if let Some(n) = &self.config.full {
                current_notification = Some(n);
            }
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

fn find_attribute_prefix<'a, 'b>(bat_name: &'a str) -> Result<&'b str, Error> {
    let entries: Vec<String> = fs::read_dir(format!("{}{}", SYS_PATH, bat_name))?
        .map(|res| {
            res.map_or("error".to_string(), |e| {
                e.file_name().into_string().unwrap()
            })
        })
        .filter(|v| v != "error")
        .collect();
    let energy_now_attribute = format!("{}_{}", ENERGY_PREFIX, NOW_ATTRIBUTE);
    let energy_full_attribute = format!("{}_{}", ENERGY_PREFIX, FULL_ATTRIBUTE);
    let energy_full_design_attribute = format!("{}_{}", ENERGY_PREFIX, FULL_DESIGN_ATTRIBUTE);
    let charge_now_attribute = format!("{}_{}", CHARGE_PREFIX, NOW_ATTRIBUTE);
    let charge_full_attribute = format!("{}_{}", CHARGE_PREFIX, FULL_ATTRIBUTE);
    let charge_full_design_attribute = format!("{}_{}", CHARGE_PREFIX, FULL_DESIGN_ATTRIBUTE);
    let mut unit = None;
    if entries.iter().any(|v| v == &energy_now_attribute)
        && entries.iter().any(|v| v == &energy_full_attribute)
        && entries.iter().any(|v| v == &energy_full_design_attribute)
    {
        unit = Some(ENERGY_PREFIX);
    } else if entries.iter().any(|v| v == &charge_now_attribute)
        && entries.iter().any(|v| v == &charge_full_attribute)
        && entries.iter().any(|v| v == &charge_full_design_attribute)
    {
        unit = Some(CHARGE_PREFIX);
    }
    unit.ok_or_else(|| {
        Error::new(format!(
            "unable to find the required files under {}{}",
            SYS_PATH, BAT_NAME
        ))
    })
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
