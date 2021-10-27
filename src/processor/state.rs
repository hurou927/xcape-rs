use core::cell::Cell;
use core::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::context::Context;

#[derive(Debug)]
pub enum NoUpdateReason {
    NoRemappedKey, // no remapped key
}

pub struct KeyState {
    pub fake_keys: Vec<u8>,
    pub is_pressed: bool,
    pub will_be_used_as_modifier: bool,
}

impl KeyState {
    fn new(fake_keys: Vec<u8>) -> Self {
        KeyState {
            fake_keys,
            is_pressed: false,
            will_be_used_as_modifier: false,
        }
    }
}

pub struct State {
    auto_generated_key_flags: RefCell<[u8; 256]>,
    is_mouse_pressed: Cell<bool>,
    pub remapped_key_states: RefCell<HashMap<u8, KeyState>>,
}

impl State {
    pub fn new(ctx: &Context) -> Self {
        let key_map = ctx
            .key_map
            .iter()
            .map(|(k, v)| (*k, KeyState::new(v.clone())))
            .collect();
        State {
            auto_generated_key_flags: RefCell::new([0; 256]),
            is_mouse_pressed: Cell::new(false),
            remapped_key_states: RefCell::new(key_map),
        }
    }

    fn update_is_mouse_pressed(&self, is_pressed: bool) {
        self.is_mouse_pressed.set(is_pressed)
    }

    pub fn press_mouse(&self) {
        self.update_is_mouse_pressed(true)
    }
    pub fn release_mouse(&self) {
        self.update_is_mouse_pressed(false)
    }

    // return old value
    fn update_key_pressed(&self, key: u8, is_pressed: bool) -> Result<bool, NoUpdateReason> {
        let old_result = match self.remapped_key_states.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                let old = new.is_pressed;
                new.is_pressed = is_pressed;
                debug!("udpate key. key:{}, new:{}, old:{:?}", key, is_pressed, old);
                Ok(old)
            }
            Entry::Vacant(_) => Err(NoUpdateReason::NoRemappedKey),
        };
        old_result
    }

    pub fn press_key(&self, key: u8) -> Result<bool, NoUpdateReason> {
        self.update_key_pressed(key, true)
    }
    pub fn release_key(&self, key: u8) -> Result<(bool, bool), NoUpdateReason> {
        match self.remapped_key_states.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                let old = (new.is_pressed, new.will_be_used_as_modifier);
                debug!(
                    "Update State: key:{}, is_pressed:{}, is_used:{}",
                    key, new.is_pressed, new.will_be_used_as_modifier
                );
                new.is_pressed = false;
                new.will_be_used_as_modifier = false;

                Ok(old)
            }
            Entry::Vacant(_) => Err(NoUpdateReason::NoRemappedKey),
        }
    }

    pub fn update_all_remapped_mod_keys_to_in_used(&self) {
        for (_, key_state) in self.remapped_key_states.borrow_mut().iter_mut() {
            if key_state.is_pressed || self.is_mouse_pressed.get() {
                key_state.will_be_used_as_modifier = true;
            }
        }
    }
    pub fn mark_auto_generated_key(&self, key: u8) {
        self.auto_generated_key_flags.borrow_mut()[key as usize] += 1;
    }
    pub fn check_and_unmark_auto_generated_key(&self, key: u8) -> Result<u8, NoUpdateReason> {
        let mut auto_generated_key_flags = self.auto_generated_key_flags.borrow_mut();
        let old_flag = auto_generated_key_flags[key as usize];
        if old_flag > 0 {
            auto_generated_key_flags[key as usize] -= 1;
            debug!(
                "remove generated key. key: {}, new:{}, old:{}",
                key, auto_generated_key_flags[key as usize], old_flag
            );
            Ok(old_flag)
        } else {
            debug!("not remove generated key: {}", key);
            Err(NoUpdateReason::NoRemappedKey)
        }
    }
}
