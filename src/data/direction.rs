#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_idx(self) -> usize {
        match self {
            Direction::Up => 0,
            Direction::Down => 2,
            Direction::Left => 3,
            Direction::Right => 1,
        }
    }
}

#[allow(dead_code)]
pub const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];
