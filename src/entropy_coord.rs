use std::cmp::Ordering;

use crate::data::vector2::Vector2;

#[derive(PartialEq, Debug, Clone)]
pub struct EntropyCoord {
    pub entropy: f32,
    pub coord: Vector2,
}

impl EntropyCoord {
    pub fn new(entropy: f32, coord: Vector2) -> EntropyCoord {
        EntropyCoord { entropy, coord }
    }
}

impl Ord for EntropyCoord {
    //
    // Written this way to make clippy happy
    //
    fn cmp(&self, other: &Self) -> Ordering {
        match (self < other, self == other) {
            (true, _) => Ordering::Less,
            (false, true) => Ordering::Equal,
            (false, false) => Ordering::Greater,
        }
    }
}

impl PartialOrd for EntropyCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.entropy.partial_cmp(&self.entropy)
    }
}

impl Eq for EntropyCoord {}
