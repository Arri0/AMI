use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Kind {
    Identity,
    Linear { min: u8, max: u8 },
}

pub fn map(kind: Kind, velocity: u8) -> u8 {
    match kind {
        Kind::Identity => velocity,
        Kind::Linear { min, max } => map_linear(velocity, min, max),
    }
}

fn map_linear(velocity: u8, min: u8, max: u8) -> u8 {
    (velocity as f32 / 127.0 * (max - min) as f32).round() as u8 + min
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_linear() {
        assert_eq!(super::map_linear(0, 0, 1), 0);
        assert_eq!(super::map_linear(60, 0, 1), 0);
        assert_eq!(super::map_linear(70, 0, 1), 1);
        assert_eq!(super::map_linear(127, 0, 1), 1);
        assert_eq!(super::map_linear(90, 0, 3), 2);
        assert_eq!(super::map_linear(90, 1, 3), 2);
    }
}
