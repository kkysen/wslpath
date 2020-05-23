pub mod win_to_wsl;
pub mod wsl_to_win;

// Works in WSL only

#[derive(Clone, Copy)]
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
        use Slash::*;
        match self {
            Forward => b'/',
            Backward => b'\\',
        }
    }
}


