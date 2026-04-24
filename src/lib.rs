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
use mio::{Events, Interest, Poll, Token};
use once_cell::sync::Lazy;
use std::convert::TryFrom;
use std::fs::{self, DirEntry};
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::config::Config;
use crate::config::Notification;

const UDEV_SUBSYSTEM: &str = "power_supply";
const SYS_PATH: &str = "/sys/class/power_supply/";
const UEVENT: &str = "uevent";
const POWER_SUPPLY: &str = "POWER_SUPPLY";
const CHARGE_PREFIX: &str = "CHARGE";
const ENERGY_PREFIX: &str = "ENERGY";
const FULL_ATTRIBUTE: &str = "FULL";
const FULL_DESIGN_ATTRIBUTE: &str = "FULL_DESIGN";
const NOW_ATTRIBUTE: &str = "NOW";
const STATUS_ATTRIBUTE: &str = "POWER_SUPPLY_STATUS";
const ONLINE_ATTRIBUTE: &str = "POWER_SUPPLY_ONLINE";
const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
const APP_DIR: &str = "bato";
const CONFIG_FILE: &str = "bato.toml";

// Global application state, used to terminate the main-loop and all modules
pub static RUN: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

#[derive(Debug)]
pub struct Bato {
    #[allow(dead_code)]
    bat_name: String,
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
        let attribute_prefix = find_attribute_prefix(&uevent)?;
        debug!("found attribute prefix: {attribute_prefix}");
        let now_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, NOW_ATTRIBUTE);
        let full_attribute = format!("{}_{}_{}", POWER_SUPPLY, attribute_prefix, full_attr);
        Ok(Bato {
            bat_name,
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
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if now.is_none() && key == self.now_attribute {
                now = Some(
                    value
                        .parse()
                        .map_err(|e| anyhow!("failed to parse value for {key}: {e}"))?,
                )
            }
            if full.is_none() && key == self.full_attribute {
                full = Some(
                    value
                        .parse()
                        .map_err(|e| anyhow!("failed to parse value for {key}: {e}"))?,
                )
            }
            if status.is_none() && key == STATUS_ATTRIBUTE {
                status = Some(value.to_string());
            }
            if now.is_some() && full.is_some() && status.is_some() {
                break;
            }
        }
        if now.is_none() {
            bail!(
                "attribute '{}' not found in {}",
                self.now_attribute,
                self.uevent
            );
        }
        if full.is_none() {
            bail!(
                "attribute '{}' not found in {}",
                self.full_attribute,
                self.uevent
            );
        }
        if status.is_none() {
            bail!(
                "attribute '{}' not found in {}",
                STATUS_ATTRIBUTE,
                self.uevent
            );
        }
        Ok((now.unwrap(), full.unwrap(), status.unwrap().as_str().into()))
    }

    #[instrument(skip(self))]
    pub fn run(&mut self, tick: Duration) -> Result<()> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        const MONITOR: Token = Token(0);

        let mut socket = udev::MonitorBuilder::new()?
            .match_subsystem(UDEV_SUBSYSTEM)?
            .listen()?;

        poll.registry()
            .register(&mut socket, MONITOR, Interest::READABLE)?;

        // initial update
        self.update(None)
            .inspect_err(|e| error!("failed to update: {e}"))
            .ok();

        while RUN.load(Ordering::Relaxed) {
            match poll.poll(&mut events, Some(tick)) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    // when laptop goes into sleep poll exits with Interrupted
                    // → retry
                    debug!("poll interrupted, retrying");
                    continue;
                }
                Err(e) => {
                    error!("poll error: {}", e);
                    bail!("poll error: {}", e);
                }
            }

            if events.is_empty() {
                // poll timeout -> no event, just update
                trace!("tick");
                self.update(None)
                    .inspect_err(|e| error!("failed to update: {e}"))
                    .ok();
            } else if events
                .iter()
                .any(|e| e.token() == MONITOR && e.is_readable())
            {
                let ac_online = socket
                    .iter()
                    .filter(|e| e.sysname() == "AC")
                    .inspect(|_| info!("AC udev event"))
                    .find_map(|e| {
                        e.property_value(ONLINE_ATTRIBUTE)
                            .and_then(|v| v.to_str())
                            .and_then(|v| match v {
                                "1" => Some(true),
                                "0" => Some(false),
                                _ => None,
                            })
                    });

                if let Some(ac) = ac_online {
                    debug!("AC online: {}", ac);
                    self.update(Some(ac))
                        .inspect_err(|e| error!("failed to update: {e}"))
                        .ok();
                }
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn update(&mut self, uevent_ac: Option<bool>) -> Result<()> {
        let (energy, capacity, sysfs_status) =
            self.parse_attributes().context("parse attribute")?;
        trace!("sysfs status {}", sysfs_status.as_ref());
        let capacity = capacity as u64;
        let energy = energy as u64;
        let battery_level = u32::try_from(100_u64 * energy / capacity)
            .context("failed to calculate battery level")?;
        // When AC uevent fires (AC is plugged or unplugged),
        // sysfs is laggy and still not refreshed by driver/kernel.
        // Pre-shot battery switch on Charging/Discharging state.
        let status = uevent_ac
            .map(|ac| match ac {
                true => PsStatus::Charging,
                false => PsStatus::Discharging,
            })
            .unwrap_or(sysfs_status);
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
