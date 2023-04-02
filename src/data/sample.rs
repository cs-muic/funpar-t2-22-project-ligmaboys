use super::colour::Rgb;
use super::direction::Direction;
use super::grid2d::Grid2D;
use super::vector2::Vector2;

//
// Sample Container
//
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sample {
    pub region: Grid2D<Rgb>,
}

pub type SampleID = usize;

// Note: You can turn idx, at, etc. into a trait

impl Sample {
    pub fn new(width: i32, height: i32) -> Sample {
        Sample {
            region: Grid2D {
                width: width as usize,
                height: height as usize,
                data: Vec::with_capacity(width as usize * height as usize),
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_rotations(&self) -> Vec<Sample> {
        vec![
            self.clone(),
            self.rev_sample(),
            self.transpose_sample(),
            self.rev_sample_y(),
        ]
    }

    #[allow(dead_code)]
    pub fn rev_sample(&self) -> Sample {
        let rev_grid_data = self.region.clone().data.into_iter().rev().collect();
        Sample {
            region: Grid2D {
                width: self.region.width,
                height: self.region.height,
                data: rev_grid_data,
            },
        }
    }

    #[allow(dead_code)]
    pub fn transpose_sample(&self) -> Sample {
        let mut transposed = vec![[0u8, 0u8, 0u8]; self.region.size()];
        transpose::transpose(
            &self.region.data,
            &mut transposed,
            self.region.width,
            self.region.height,
        );
        Sample {
            region: Grid2D {
                width: self.region.width,
                height: self.region.height,
                data: transposed,
            },
        }
    }

    #[allow(dead_code)]
    pub fn rev_sample_y(&self) -> Sample {
        let transposed = self.clone().transpose_sample();
        let reversed_y = transposed
            .region
            .data
            .iter()
            .map(|[r, g, b]| [*b, *g, *r])
            .collect();
        Sample {
            region: Grid2D {
                width: self.region.width,
                height: self.region.height,
                data: reversed_y,
            },
        }
    }

    pub fn rotate(&self) -> Vec<Sample> {
        let mut rotations = vec![self.clone()];
        let mut rotate = self.clone();
        for _n in 0..3 {
            rotate = rotate.rotate_90();
            rotations.push(rotate.clone());
        }
        rotations
    }

    pub fn rotate_90(&self) -> Sample {
        let data = self.clone().transpose_sample().region.data;
        let mut rotated = data;
        rotated = (0..self.region.width)
            .flat_map(|row| {
                rotated[(row * self.region.height)..(row * self.region.height) + self.region.height]
                    .iter()
                    .copied()
                    .rev()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Sample {
            region: Grid2D {
                width: self.region.width,
                height: self.region.height,
                data: rotated,
            },
        }
    }

    #[allow(dead_code)]
    pub fn get_top_left_pixel(&self) -> Rgb {
        *self.region.get(Vector2 { x: 0, y: 0 }).unwrap()
    }

    //
    // Retrieve index position from Vector2 position
    //
    #[allow(dead_code)]
    pub fn idx(&self, at: Vector2) -> usize {
        self.region.idx(at).unwrap()
    }

    //
    // Retrieve colour from Vector2 position
    //
    pub fn at(&self, at: Vector2) -> Rgb {
        *self.region.get(at).unwrap()
    }

    pub fn compatible(&self, other: &Sample, direction: Direction) -> bool {
        if other.region.width != self.region.width || other.region.height != self.region.height {
            return false;
        }

        let (xs, ys, offset) = match direction {
            // Bottom of A, top of B
            Direction::Down => (
                (0..self.region.width),
                (0..self.region.height - 1),
                Vector2 { x: 0, y: 1 },
            ),
            // Top of A, bottom of B
            Direction::Up => (
                (0..self.region.width),
                (1..self.region.height),
                Vector2 { x: 0, y: -1 },
            ),
            // Left of A, right of B
            Direction::Left => (
                (1..self.region.width),
                (0..self.region.height),
                Vector2 { x: -1, y: 0 },
            ),
            // Right of A, left of B
            Direction::Right => (
                (0..self.region.width - 1),
                (0..self.region.height),
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
    fn test_rotate() {
        let s1: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };
        let s2: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [136, 136, 255],
                    [136, 136, 255],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                ]
                .to_vec(),
            },
        };
        let s3: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
                    [136, 136, 255],
                    [136, 136, 255],
                    [136, 136, 255],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                ]
                .to_vec(),
            },
        };
        assert_eq!(s1.rotate_90(), s2);
        assert_eq!(s1.rotate_90().rotate_90(), s3);
    }

    #[test]
    fn test_eq() {
        let s1: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };

        let s2: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };
        assert!(s1.compatible(&s2, Direction::Right));
    }

    #[test]
    fn test_neq() {
        let s1 = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };
        let s2 = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };
        assert!(!&s1.compatible(&s2, Direction::Up));
        assert!(!&s1.compatible(&s2, Direction::Right));
        assert!(!&s1.compatible(&s2, Direction::Left));
        assert!(!&s1.compatible(&s2, Direction::Down));
    }
    #[test]
    fn test_top_left() {
        let s1: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };

        let s2: Sample = Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: [
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
            },
        };
        assert_eq!(s2.get_top_left_pixel(), [136, 136, 255]);
        assert_eq!(s1.get_top_left_pixel(), [0, 0, 0]);
    }
}
