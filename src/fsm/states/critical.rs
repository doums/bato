// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tracing::{debug, info, instrument, trace};

use super::fsm::FsmState;
use super::fsm_impl::{Data, PsStatus, State};
use crate::Config;

pub struct CriticalState(pub Config);

impl FsmState<State, Data> for CriticalState {
    #[instrument(skip_all, fields(current = "critical"))]
    fn enter(&self, _: &Data) {
        trace!("enter");
        if let Some(n) = self.0.critical.as_ref() {
            info!("sending notification");
            crate::notify(n).ok();
        }
    }

    #[instrument(skip_all, fields(current = "critical"))]
    fn next_state(&self, data: &Data) -> Option<State> {
        if data.status == PsStatus::Charging {
            debug!("next_state {}", State::Charging);
            return Some(State::Charging);
        }
        None
    }

    #[instrument(skip_all, fields(current = "critical"))]
    fn exit(&self, _data: &Data) {
        trace!("exit");
    }
}
