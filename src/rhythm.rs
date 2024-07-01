use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rhythm {
    pub num_beats: u8,
    pub num_divs: u8,
}

impl Rhythm {
    pub fn num_slots(&self) -> usize {
        self.num_beats as usize * self.num_divs as usize
    }
}

impl Default for Rhythm {
    fn default() -> Self {
        Self {
            num_beats: 4,
            num_divs: 4,
        }
    }
}