use super::colour::Rgb;
use super::direction::Direction;
use super::vector2::Vector2;

//
// Sample Container
//
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sample {
    pub width: u32,
    pub height: u32,
    pub region: Vec<Rgb>,
}

pub type SampleID = usize;

// Note: You can turn idx, at, etc. into a trait

impl Sample {
    pub fn new(width: u32, height: u32) -> Sample {
        Sample {
            width,
            height,
            region: Vec::new(),
        }
    }

    //
    // Retrieve index position from Vector2 position
    //
    pub fn idx(&self, at: Vector2) -> usize {
        (at.y as u32 * self.width) as usize + at.x as usize
    }

    //
    // Retrieve colour from Vector2 position
    //
    pub fn at(&self, at: Vector2) -> Rgb {
        let idx = self.idx(at);
        self.region[idx]
    }

    pub fn compatible(&self, other: &Sample, direction: Direction) -> bool {
        if other.width != self.width || other.height != self.height {
            return false;
        }

        let (xs, ys, offset) = match direction {
            // Bottom of A, top of B
            Direction::Down => (
                (0..self.width),
                (0..self.height - 1),
                Vector2 { x: 0, y: 1 },
            ),
            // Top of A, bottom of B
            Direction::Up => ((0..self.width), (1..self.height), Vector2 { x: 0, y: -1 }),
            // Left of A, right of B
            Direction::Left => ((1..self.width), (0..self.height), Vector2 { x: -1, y: 0 }),
            // Right of A, left of B
            Direction::Right => (
                (0..self.width - 1),
                (0..self.height),
                Vector2 { x: 1, y: 0 },
            ),
        };

        ys.into_iter().all(|y| {
            xs.clone().all(|x| {
                let pos = Vector2 {
                    x: x as i32,
                    y: y as i32,
                };
                self.at(pos + offset) == other.at(pos)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let s1: Sample = Sample {
            width: 3,
            height: 3,
            region: [
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
            .to_vec(),
        };

        let s2 = Sample {
            width: 3,
            height: 3,
            region: [
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
            .to_vec(),
        };
        assert!(s1.compatible(&s2, Direction::Right));
    }

    #[test]
    fn test_neq() {
        let s1 = Sample {
            width: 3,
            height: 3,
            region: [
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
            .to_vec(),
        };
        let s2 = Sample {
            width: 3,
            height: 3,
            region: [
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
            ]
            .to_vec(),
        };
        assert!(!&s1.compatible(&s2, Direction::Up));
        assert!(!&s1.compatible(&s2, Direction::Right));
        assert!(!&s1.compatible(&s2, Direction::Left));
        assert!(!&s1.compatible(&s2, Direction::Down));
    }
}
