// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, hash::Hash};

pub type StateMap<K, D> = HashMap<K, Box<dyn FsmState<K, D>>>;

pub struct Fsm<K, D>
where
    K: Eq + Hash,
{
    current_state: K,
    states: StateMap<K, D>,
}

impl<K, D> Fsm<K, D>
where
    K: Eq + Hash,
{
    pub fn new(init_state: K, states: StateMap<K, D>) -> Self {
        Fsm {
            current_state: init_state,
            states,
        }
    }

    fn set_state(&mut self, new_state: K, data: &mut D) {
        self.states.get_mut(&self.current_state).unwrap().exit(data);
        self.states.get_mut(&new_state).unwrap().enter(data);
        self.current_state = new_state;
    }

    pub fn shift(&mut self, data: &mut D) {
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
    fn enter(&mut self, data: &mut D);
    fn next_state(&self, data: &mut D) -> Option<K>;
    fn exit(&mut self, data: &mut D);
}
