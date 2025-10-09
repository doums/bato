// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, instrument, trace};

use super::fsm::FsmState;
use super::fsm_impl::{Data, PsStatus, State};
use crate::Config;

pub struct FullState(pub Config);

impl FsmState<State, Data> for FullState {
    #[instrument(skip_all, fields(current = "full"))]
    fn enter(&self, _: &Data) {
        trace!("enter");
        if let Some(n) = self.0.full.as_ref() {
            info!("sending notification");
            crate::notify(n).ok();
        }
    }

    #[instrument(skip_all, fields(current = "full"))]
    fn next_state(&self, data: &Data) -> Option<State> {
        let state = match data.status {
            PsStatus::Charging => Some(State::Charging),
            PsStatus::NotCharging => Some(State::NotCharging),
            PsStatus::Discharging => Some(State::Discharging),
            _ => None,
        };
        state.inspect(|s| debug!("next_state {s}"))
    }

    #[instrument(skip_all, fields(current = "full"))]
    fn exit(&self, _data: &Data) {
        trace!("exit");
    }
}
