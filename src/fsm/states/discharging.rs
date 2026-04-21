// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, instrument, trace};

use super::fsm::FsmState;
use super::fsm_impl::{Data, PsStatus, State};
use crate::Config;

pub struct DischargingState(pub Config);

impl FsmState<State, Data> for DischargingState {
    #[instrument(skip_all, fields(current = "discharging"))]
    fn enter(&self, _: &Data) {
        trace!("enter");
        if let Some(n) = self.0.discharging.as_ref() {
            info!("sending notification");
            crate::notify(n).ok();
        }
    }

    #[instrument(skip_all, fields(current = "discharging"))]
    fn next_state(&self, data: &Data) -> Option<State> {
        let state = match (data.status, data.current_level) {
            (PsStatus::Charging, _) => Some(State::Charging),
            (PsStatus::Full, _) => Some(State::Full), // add this just in case
            (PsStatus::NotCharging, _) => Some(State::NotCharging),
            (PsStatus::Discharging, l) if l <= self.0.critical_level => Some(State::Critical),
            (PsStatus::Discharging, l) if l <= self.0.low_level => Some(State::Low),
            _ => None,
        };
        state.inspect(|s| debug!("next_state {s}"))
    }

    #[instrument(skip_all, fields(current = "discharging"))]
    fn exit(&self, _data: &Data) {
        trace!("exit");
    }
}

impl std::fmt::Debug for DischargingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DischargingState")
    }
}
