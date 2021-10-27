use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    DEBUG,
    NORMAL,
}

#[derive(Debug, Clone)]
pub struct Context {
    mode: Mode,
    pub timeout_sec: Option<u64>,
    pub key_map: HashMap<u8, Vec<u8>>,
}

impl Context {
    pub fn new(is_debug: bool, timeout_sec: Option<u64>, key_map: HashMap<u8, Vec<u8>>) -> Self {
        Context {
            mode: if is_debug { Mode::DEBUG } else { Mode::NORMAL },
            timeout_sec,
            key_map,
        }
    }

    pub fn is_debug_mode(&self) -> bool {
        match self.mode {
            Mode::DEBUG => true,
            _ => false,
        }
    }
}
