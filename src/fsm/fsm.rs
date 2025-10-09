// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Display};
use std::{collections::HashMap, hash::Hash};
use tracing::{debug, info, instrument};

pub type StateMap<K, D> = HashMap<K, Box<dyn FsmState<K, D>>>;

pub struct Fsm<K, D>
where
    K: Eq + Hash + Display + Debug,
    D: Display + Debug,
{
    current_state: K,
    states: StateMap<K, D>,
}

impl<K, D> Fsm<K, D>
where
    K: Eq + Hash + Display + Debug,
    D: Display + Debug,
{
    #[instrument(skip_all)]
    pub fn new(init_state: K, states: StateMap<K, D>) -> Self {
        debug!("fsm init {init_state}, states count #{}", states.len());
        Fsm {
            current_state: init_state,
            states,
        }
    }

    #[instrument(skip_all)]
    fn set_state(&mut self, new_state: K, data: &D) {
        info!("new state {new_state}");
        self.states.get_mut(&self.current_state).unwrap().exit(data);
        self.states.get_mut(&new_state).unwrap().enter(data);
        self.current_state = new_state;
    }

    #[instrument(skip_all)]
    pub fn shift(&mut self, data: &D) {
        debug!("shift {data}");
        if let Some(next_state) = self
            .states
            .get_mut(&self.current_state)
            .unwrap()
            .next_state(data)
        {
            self.set_state(next_state, data);
        }
    }
}

pub trait FsmState<K, D>
where
    K: Eq + Hash,
{
    fn enter(&self, data: &D);
    fn next_state(&self, data: &D) -> Option<K>;
    fn exit(&self, data: &D);
}
