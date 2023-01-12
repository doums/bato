// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod battery;
mod error;
mod fsm;
mod notify;
use battery::{Data, State};
use error::Error;
use fsm::Fsm;
use notify::{close_libnotilus, init_libnotilus, NotifyNotification};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::ptr;

const APP_NAME: &str = "bato";
const SYS_PATH: &str = "/sys/class/power_supply/";
const BAT_NAME: &str = "BAT0";
const UEVENT: &str = "uevent";
const POWER_SUPPLY: &str = "POWER_SUPPLY";
const CHARGE_PREFIX: &str = "CHARGE";
const ENERGY_PREFIX: &str = "ENERGY";
const FULL_ATTRIBUTE: &str = "FULL";
const FULL_DESIGN_ATTRIBUTE: &str = "FULL_DESIGN";
const NOW_ATTRIBUTE: &str = "NOW";
const STATUS_ATTRIBUTE: &str = "POWER_SUPPLY_STATUS";

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(C)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationConfig {
    summary: String,
    body: Option<String>,
    icon: Option<String>,
    urgency: Option<Urgency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub tick_rate: Option<u32>,
    bat_name: Option<String>,
    low_level: u32,
    critical_level: u32,
    full_design: Option<bool>,
    critical: Option<NotificationConfig>,
    low: Option<NotificationConfig>,
    full: Option<NotificationConfig>,
    charging: Option<NotificationConfig>,
    discharging: Option<NotificationConfig>,
}

pub struct Notification {
    summary: CString,
    body: Option<CString>,
    icon: Option<CString>,
    urgency: Option<Urgency>,
}

impl TryFrom<NotificationConfig> for Notification {
    type Error = Error;

    fn try_from(config: NotificationConfig) -> Result<Self, Self::Error> {
        let summary = CString::new(config.summary).map_err(|_| "NulError")?;
        let body = config
            .body
            .map(|s| CString::new(s).map_err(|_| "NulError"))
            .transpose()?;
        let icon = config
            .icon
            .map(|s| CString::new(s).map_err(|_| "NulError"))
            .transpose()?;
        Ok(Notification {
            summary,
            body,
            icon,
            urgency: config.urgency,
        })
    }
}

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

impl Config {
    pub fn new() -> Result<Self, Error> {
        let home = env::var("HOME").map_err(|err| format!("environment variable HOME, {}", err))?;
        let config_path =
            env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));
        let content = fs::read_to_string(format!("{}/bato/bato.yaml", config_path))
            .map_err(|err| format!("while reading the config file, {}", err))?;
        let user_config: UserConfig = serde_yaml::from_str(&content)
            .map_err(|err| format!("while deserializing the config file, {}", err))?;
        let critical = user_config
            .critical
            .map(Notification::try_from)
            .transpose()?;

        Ok(Config {
            tick_rate: user_config.tick_rate,
            bat_name: user_config.bat_name,
            low_level: user_config.low_level,
            critical_level: user_config.critical_level,
            full_design: user_config.full_design,
            critical,
            low: user_config.low.map(Notification::try_from).transpose()?,
            full: user_config.full.map(Notification::try_from).transpose()?,
            charging: user_config
                .charging
                .map(Notification::try_from)
                .transpose()?,
            discharging: user_config
                .discharging
                .map(Notification::try_from)
                .transpose()?,
        })
    }

    pub fn normalize(&mut self) -> &Self {
        if let Some(v) = &mut self.critical {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Critical)
            }
        }
        if let Some(v) = &mut self.low {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Normal)
            }
        }
        if let Some(v) = &mut self.full {
            if v.urgency.is_none() {
                v.urgency = Some(Urgency::Normal)
            }
        }
        self
    }
}

pub struct Bato<'data> {
    uevent: String,
    now_attribute: String,
    full_attribute: String,
    notification: *mut NotifyNotification,
    fsm: Fsm<State, Data<'data>>,
}

impl<'data> Bato<'data> {
    pub fn with_config(config: &Config) -> Result<Self, Error> {
        let bat_name = if let Some(v) = &config.bat_name {
            String::from(v)
        } else {
            String::from(BAT_NAME)
        };
        let mut full_design = true;
        if let Some(v) = config.full_design {
            full_design = v;
        }
        let full_attr = match full_design {
            true => FULL_DESIGN_ATTRIBUTE,
            false => FULL_ATTRIBUTE,
        };
        let uevent = format!("{}{}/{}", SYS_PATH, &bat_name, UEVENT);
        let attribute_prefix = find_attribute_prefix(&uevent)?;
        let now_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, NOW_ATTRIBUTE);
        let full_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, full_attr);
        Ok(Bato {
            uevent,
            now_attribute,
            full_attribute,
            notification: ptr::null_mut(),
            fsm: battery::create_fsm(),
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

    fn parse_attributes(&self) -> Result<(i32, i32, String), Error> {
        let file = File::open(&self.uevent)?;
        let f = BufReader::new(file);
        let mut now = None;
        let mut full = None;
        let mut status = None;
        for line in f.lines() {
            if now.is_none() {
                now = parse_attribute(&line, &self.now_attribute);
            }
            if full.is_none() {
                full = parse_attribute(&line, &self.full_attribute);
            }
            if status.is_none() {
                status = parse_status(&line);
            }
        }
        if now.is_none() || full.is_none() || status.is_none() {
            return Err(Error::new(format!(
                "unable to parse the required attributes in {}",
                self.uevent
            )));
        }
        Ok((now.unwrap(), full.unwrap(), status.unwrap()))
    }

    pub fn update(&mut self, config: &'data Config) -> Result<(), Error> {
        let (energy, capacity, status) = self.parse_attributes()?;
        let capacity = capacity as u64;
        let energy = energy as u64;
        let battery_level = u32::try_from(100_u64 * energy / capacity)?;
        let mut data = Data {
            current_level: battery_level,
            status,
            low_level: config.low_level,
            critical_level: config.critical_level,
            critical: config.critical.as_ref(),
            low: config.low.as_ref(),
            full: config.full.as_ref(),
            charging: config.charging.as_ref(),
            discharging: config.discharging.as_ref(),
            notification: self.notification,
        };
        self.fsm.shift(&mut data);
        Ok(())
    }

    pub fn close(&mut self) {
        close_libnotilus(self.notification)
    }
}

fn parse_attribute(line: &io::Result<String>, attribute: &str) -> Option<i32> {
    if let Ok(l) = line {
        if l.starts_with(attribute) {
            let s = l.split('=').nth(1);
            if let Some(v) = s {
                return v.parse::<i32>().ok();
            }
        }
    }
    None
}

fn parse_status(line: &io::Result<String>) -> Option<String> {
    if let Ok(l) = line {
        if l.starts_with(STATUS_ATTRIBUTE) {
            return l.split('=').nth(1).map(|s| s.to_string());
        }
    }
    None
}

fn find_attribute_prefix<'a, 'b>(path: &'a str) -> Result<&'b str, Error> {
    let content = fs::read_to_string(path)?;
    let mut unit = None;
    if content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, FULL_DESIGN_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, FULL_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, NOW_ATTRIBUTE
    )) {
        unit = Some(ENERGY_PREFIX);
    } else if content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, CHARGE_PREFIX, FULL_DESIGN_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, CHARGE_PREFIX, FULL_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, CHARGE_PREFIX, NOW_ATTRIBUTE
    )) {
        unit = Some(CHARGE_PREFIX);
    }
    unit.ok_or_else(|| {
        Error::new(format!(
            "unable to find the required attributes in {}",
            path
        ))
    })
}
