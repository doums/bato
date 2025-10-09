// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod cli;
mod config;
mod fsm;
pub mod signal;
pub mod trace;
mod util;

use anyhow::{Context, Result, anyhow, bail};
use fsm::{Data, Fsm, PsStatus, State};
use once_cell::sync::Lazy;
use std::convert::TryFrom;
use std::fs::{self, DirEntry};
use std::sync::atomic::AtomicBool;
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::config::Config;
use crate::config::Notification;

const SYS_PATH: &str = "/sys/class/power_supply/";
const UEVENT: &str = "uevent";
const POWER_SUPPLY: &str = "POWER_SUPPLY";
const CHARGE_PREFIX: &str = "CHARGE";
const ENERGY_PREFIX: &str = "ENERGY";
const FULL_ATTRIBUTE: &str = "FULL";
const FULL_DESIGN_ATTRIBUTE: &str = "FULL_DESIGN";
const NOW_ATTRIBUTE: &str = "NOW";
const STATUS_ATTRIBUTE: &str = "POWER_SUPPLY_STATUS";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const APP_DIR: &str = "bato";
const CONFIG_FILE: &str = "bato.toml";

// Global application state, used to terminate the main-loop and all modules
pub static RUN: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

pub struct Bato {
    uevent: String,
    now_attribute: String,
    full_attribute: String,
    fsm: Fsm<State, Data>,
}

// check if the given battery is present
#[instrument]
pub fn check_system_path(battery: &Option<String>) -> Result<()> {
    // overkill check but for the sake of…
    util::check_dir(SYS_PATH).inspect_err(|_| trace!("Anomalous Materials"))?;

    if let Some(name) = battery {
        util::check_dir(&format!("{SYS_PATH}{name}"))
            .inspect_err(|e| error!("no battery found with name {name}: {e}"))?;
    }
    Ok(())
}

#[instrument]
pub fn notify(to_send: &Notification) -> Result<()> {
    let mut ntf = notify_rust::Notification::new()
        .summary(&to_send.summary)
        .finalize();
    if let Some(body) = to_send.body.as_ref() {
        ntf.body(body);
    }
    if let Some(icon) = to_send.icon.as_ref() {
        ntf.icon(icon);
    }
    if let Some(urgency) = to_send.urgency.as_ref() {
        ntf.urgency(notify_rust::Urgency::from(urgency));
    }
    debug!("notify show");
    ntf.show()
        .inspect_err(|e| error!("failed to show notification: {e}"))?;
    Ok(())
}

// find a battery in `/sys/class/power_supply/`
#[instrument]
pub fn find_battery() -> Result<Option<String>> {
    let entries: Vec<DirEntry> = fs::read_dir(SYS_PATH)
        .inspect_err(|e| error!("failed to read dir {}: {}", SYS_PATH, e))?
        .filter_map(|entry| {
            entry
                .inspect_err(|e| warn!("failed to read entry in {SYS_PATH}: {e}"))
                .ok()
        })
        .collect();
    // TODO improve, use and look for POWER_SUPPLY_TYPE=Battery attribute
    let entry = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            e.file_name()
                .into_string()
                .inspect_err(|e| warn!("failed to convert OsString: {}", e.display()))
                .ok()
        })
        .find(|dir_name| dir_name.to_lowercase().starts_with("bat"))
        .inspect(|e| debug!("found battery {e}"));
    Ok(entry)
}

impl Bato {
    #[instrument(skip_all)]
    pub fn with_config(config: Config) -> Result<Self> {
        check_system_path(&config.bat_name)?;
        let bat_name: String = if let Some(name) = &config.bat_name {
            name.clone()
        } else {
            debug!("no battery name provided");
            find_battery()?.ok_or_else(|| {
                error!("no battery found in {SYS_PATH}");
                anyhow!("no battery found in {SYS_PATH}")
            })?
        };
        info!("using battery {bat_name}");
        let full_attr = match config.full_design {
            true => FULL_DESIGN_ATTRIBUTE,
            false => FULL_ATTRIBUTE,
        };
        let uevent = format!("{}{}/{}", SYS_PATH, &bat_name, UEVENT);
        debug!("battery file {uevent}");
        let attribute_prefix = find_attribute_prefix(&uevent)?;
        debug!("found attribute prefix: {attribute_prefix}");
        let now_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, NOW_ATTRIBUTE);
        let full_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, full_attr);
        debug!("now attribute: {now_attribute}");
        debug!("full attribute: {full_attribute}");
        Ok(Bato {
            uevent,
            now_attribute,
            full_attribute,
            fsm: fsm::create(config),
        })
    }

    /// Read and parse attributes value in /sys/class/power_supply/<BAT_NAME>/uevent
    #[instrument(skip(self))]
    fn parse_attributes(&self) -> Result<(i32, i32, PsStatus)> {
        let mut now = None;
        let mut full = None;
        let mut status = None;
        for line in fs::read_to_string(&self.uevent)
            .inspect_err(|e| error!("failed to read {}: {e}", self.uevent))?
            .lines()
        {
            if now.is_none() && line.starts_with(&self.now_attribute) {
                now = line.split('=').nth(1).and_then(|v| v.parse().ok());
            }
            if full.is_none() && line.starts_with(&self.full_attribute) {
                full = line.split('=').nth(1).and_then(|v| v.parse().ok());
            }
            if status.is_none() && line.starts_with(STATUS_ATTRIBUTE) {
                status = line.split('=').nth(1).map(|s| s.to_string());
            }
        }
        if now.is_none() {
            bail!("failed to parse attribute now in {}", self.uevent);
        }
        if full.is_none() {
            bail!("failed to parse attribute full in {}", self.uevent);
        }
        if status.is_none() {
            bail!("failed to parse attribute status in {}", self.uevent);
        }
        Ok((now.unwrap(), full.unwrap(), status.unwrap().as_str().into()))
    }

    #[instrument(skip_all)]
    pub fn update(&mut self) -> Result<()> {
        let (energy, capacity, status) = self.parse_attributes().context("parse attribute")?;
        let capacity = capacity as u64;
        let energy = energy as u64;
        let battery_level = u32::try_from(100_u64 * energy / capacity)
            .context("failed to calculate battery level")?;
        let data = Data {
            current_level: battery_level,
            status,
        };
        debug!("update: {}", data);
        self.fsm.shift(&data);
        Ok(())
    }
}

#[instrument]
fn find_attribute_prefix<'b>(path: &str) -> Result<&'b str> {
    let content = fs::read_to_string(path)?;
    let unit = if content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, FULL_DESIGN_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, FULL_ATTRIBUTE
    )) && content.contains(&format!(
        "{}_{}_{}=",
        POWER_SUPPLY, ENERGY_PREFIX, NOW_ATTRIBUTE
    )) {
        debug!("found attribute prefix {ENERGY_PREFIX}");
        Some(ENERGY_PREFIX)
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
        debug!("found attribute prefix {CHARGE_PREFIX}");
        Some(CHARGE_PREFIX)
    } else {
        None
    };
    unit.ok_or_else(|| {
        error!("unable to find attribute in {path}");
        anyhow!("unable to find attribute in {path}")
    })
}
