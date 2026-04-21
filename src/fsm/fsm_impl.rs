// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Display, Formatter};
use std::{collections::HashMap, hash::Hash};
use tracing::warn;

use super::states::*;

use super::fsm::{Fsm, StateMap};
use crate::Config;

// https://github.com/torvalds/linux/blob/5472d60c129f75282d94ae5ad072ee6dfb7c7246/include/linux/power_supply.h#L36
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, strum::AsRefStr)]
pub enum PsStatus {
    Unknown,
    Full,
    NotCharging,
    Charging,
    Discharging,
}

#[derive(Hash, Eq, PartialEq, Debug, strum::Display)]
pub enum State {
    Charging,
    Discharging,
    NotCharging,
    Full,
    Low,
    Critical,
}

#[derive(Debug)]
pub struct Data {
    pub current_level: u32,
    pub status: PsStatus,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "level {}, status {}",
            self.current_level,
            self.status.as_ref()
        )
    }
}

pub fn create(config: Config) -> Fsm<State, Data> {
    let mut states: StateMap<State, Data> = HashMap::new();
    states.insert(State::Full, Box::new(FullState(config.clone())));
    states.insert(State::NotCharging, Box::new(NotChargingState));
    states.insert(State::Charging, Box::new(ChargingState(config.clone())));
    states.insert(
        State::Discharging,
        Box::new(DischargingState(config.clone())),
    );
    states.insert(State::Low, Box::new(LowState(config.clone())));
    states.insert(State::Critical, Box::new(CriticalState(config.clone())));
    Fsm::new(State::Discharging, states)
}

impl From<&str> for PsStatus {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "full" => PsStatus::Full,
            "charging" => PsStatus::Charging,
            "discharging" => PsStatus::Discharging,
            "not charging" => PsStatus::NotCharging,
            _ => {
                warn!("unknown status {value}");
                PsStatus::Unknown
            }
        }
    }
}
