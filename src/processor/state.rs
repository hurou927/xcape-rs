use core::cell::Cell;
use core::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::context::Context;

pub struct KeyState {
    pub fake_keys: Vec<u8>,
    pub is_pressed: bool,
    pub is_used: bool,
}

impl KeyState {
    fn new(fake_keys: Vec<u8>) -> Self {
        KeyState {
            fake_keys: fake_keys,
            is_pressed: false,
            is_used: false,
        }
    }
}

pub struct State {
    is_mouse_pressed: Cell<bool>,
    pub key_map: RefCell<HashMap<u8, KeyState>>,
}

impl State {
    pub fn new(ctx: &Context) -> Self {
        let key_map = ctx
            .key_map
            .iter()
            .map(|(k, v)| (*k, KeyState::new(v.clone())))
            .collect();
        State {
            is_mouse_pressed: Cell::new(false),
            key_map: RefCell::new(key_map),
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
    fn update_key_pressed(&self, key: u8, is_pressed: bool) -> Option<bool> {
        let old = match self.key_map.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                let old = new.is_pressed;
                new.is_pressed = is_pressed;
                Some(old)
            }
            Entry::Vacant(_) => None,
        };
        debug!("udpate key. key:{}, new:{}, old:{:?}", key, is_pressed, old);
        old
    }

    pub fn press_key(&self, key: u8) -> Option<bool> {
        self.update_key_pressed(key, true)
    }
    pub fn release_key(&self, key: u8) -> Option<(bool, bool)> {
        match self.key_map.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                let old = (new.is_pressed, new.is_used);
                debug!(
                    "Update State: key:{}, is_pressed:{}, is_used:{}",
                    key, new.is_pressed, new.is_used
                );
                new.is_pressed = false;
                new.is_used = false;

                Some(old)
            }
            Entry::Vacant(_) => None,
        }
    }

    pub fn update_key_used(&self, is_used: bool) {
        for (_, val) in self.key_map.borrow_mut().iter_mut() {
            if val.is_pressed {
                val.is_used = is_used;
            }
        }
    }
}
