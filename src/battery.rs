// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    fsm::{Fsm, FsmState, StateMap},
    notify::{send, NotifyNotification},
    Notification,
};
use std::{collections::HashMap, hash::Hash};

struct ChargingState;
struct DischargingState;
struct FullState;
struct LowState;
struct CriticalState;

#[derive(Hash, Eq, PartialEq)]
pub enum State {
    Charging,
    Discharging,
    Full,
    Low,
    Critical,
}

pub struct Data<'data> {
    pub current_level: u32,
    pub status: String,
    pub low_level: u32,
    pub critical_level: u32,
    pub critical: Option<&'data Notification>,
    pub low: Option<&'data Notification>,
    pub full: Option<&'data Notification>,
    pub charging: Option<&'data Notification>,
    pub discharging: Option<&'data Notification>,
    pub notification: *mut NotifyNotification,
}

impl<'data> FsmState<State, Data<'data>> for ChargingState {
    fn enter(&mut self, data: &mut Data) {
        if let Some(n) = data.charging {
            send(data.notification, n);
        }
    }

    fn next_state(&self, data: &mut Data) -> Option<State> {
        match (data.status.as_str(), data.current_level) {
            ("Full", _) => Some(State::Full),
            ("Discharging", l) if l <= data.critical_level => Some(State::Critical),
            ("Discharging", l) if l <= data.low_level => Some(State::Low),
            ("Discharging", _) => Some(State::Discharging),
            _ => None,
        }
    }

    fn exit(&mut self, _data: &mut Data) {}
}

impl<'data> FsmState<State, Data<'data>> for DischargingState {
    fn enter(&mut self, data: &mut Data) {
        if let Some(n) = data.discharging {
            send(data.notification, n);
        }
    }

    fn next_state(&self, data: &mut Data) -> Option<State> {
        match (data.status.as_str(), data.current_level) {
            ("Charging", _) => Some(State::Charging),
            ("Full", _) => Some(State::Full), // add this just in case
            ("Discharging", l) if l <= data.critical_level => Some(State::Critical),
            ("Discharging", l) if l <= data.low_level => Some(State::Low),
            _ => None,
        }
    }

    fn exit(&mut self, _data: &mut Data) {}
}

impl<'data> FsmState<State, Data<'data>> for FullState {
    fn enter(&mut self, data: &mut Data) {
        if let Some(n) = data.full {
            send(data.notification, n);
        }
    }

    fn next_state(&self, data: &mut Data) -> Option<State> {
        match data.status.as_str() {
            "Charging" => Some(State::Charging),
            "Discharging" => Some(State::Discharging),
            _ => None,
        }
    }

    fn exit(&mut self, _data: &mut Data) {}
}

impl<'data> FsmState<State, Data<'data>> for LowState {
    fn enter(&mut self, data: &mut Data) {
        if let Some(n) = data.low {
            send(data.notification, n);
        }
    }

    fn next_state(&self, data: &mut Data) -> Option<State> {
        match (data.status.as_str(), data.current_level) {
            ("Charging", _) => Some(State::Charging),
            ("Discharging", l) if l <= data.critical_level => Some(State::Critical),
            _ => None,
        }
    }

    fn exit(&mut self, _data: &mut Data) {}
}

impl<'data> FsmState<State, Data<'data>> for CriticalState {
    fn enter(&mut self, data: &mut Data) {
        if let Some(n) = data.critical {
            send(data.notification, n);
        }
    }

    fn next_state(&self, data: &mut Data) -> Option<State> {
        if data.status == "Charging" {
            return Some(State::Charging);
        }
        None
    }

    fn exit(&mut self, _data: &mut Data) {}
}

pub fn create_fsm<'data>() -> Fsm<State, Data<'data>> {
    let mut states: StateMap<State, Data> = HashMap::new();
    states.insert(State::Full, Box::new(FullState));
    states.insert(State::Charging, Box::new(ChargingState));
    states.insert(State::Discharging, Box::new(DischargingState));
    states.insert(State::Low, Box::new(LowState));
    states.insert(State::Critical, Box::new(CriticalState));
    Fsm::new(State::Discharging, states)
}
