pub mod win_to_wsl;
pub mod wsl_to_win;

// Works in WSL only

pub enum Slash {
    Forward,
    Backward,
}

impl Default for Slash {
    fn default() -> Self {
        Slash::Backward
    }
}

impl Slash {
    pub fn value(&self) -> u8 {
        match self {
            Slash::Forward => b'/',
            Slash::Backward => b'\\',
        }
    }
}


