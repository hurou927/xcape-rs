

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    DEBUG,
    NORMAL
}

#[derive(Debug, Copy, Clone)]
pub struct Context {
    mode: Mode,

}

impl Context {
    pub fn new(
        is_debug: bool
    ) -> Self {
        Context {
            mode: if is_debug { Mode::DEBUG } else { Mode::NORMAL }
        }
    }
    
    pub fn is_debug_mode(&self) -> bool {
        match self.mode {
            Mode::DEBUG => true,
            _ => false
        }
    }


}
