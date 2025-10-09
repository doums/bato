// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, instrument, trace};

use super::fsm::FsmState;
use super::fsm_impl::{Data, PsStatus, State};

pub struct NotChargingState;

impl FsmState<State, Data> for NotChargingState {
    #[instrument(skip_all, fields(current = "not_charging"))]
    fn enter(&self, _: &Data) {
        trace!("enter");
    }

    #[instrument(skip_all, fields(current = "not_charging"))]
    fn next_state(&self, data: &Data) -> Option<State> {
        let state = match data.status {
            PsStatus::Charging => Some(State::Charging),
            PsStatus::Discharging => Some(State::Discharging),
            PsStatus::Full => Some(State::Full),
            _ => None,
        };
        state.inspect(|s| debug!("next_state {s}"))
    }

    #[instrument(skip_all, fields(current = "not_charging"))]
    fn exit(&self, _data: &Data) {
        trace!("exit");
    }
}
