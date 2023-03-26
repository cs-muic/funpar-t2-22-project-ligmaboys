use std::ops::{Add, Sub};

use super::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,

}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Vector2 {
    pub fn neighbor(&self, direction: Direction) -> Vector2 {
        match direction {
            Direction::Down => Vector2 {
                y: self.y + 1,
                x: self.x,
            },
            Direction::Left => Vector2 {
                y: self.y,
                x: self.x - 1,
            },
            Direction::Right => Vector2 {
                y: self.y,
                x: self.x + 1,
            },
            Direction::Up => Vector2 {
                y: self.y - 1,
                x: self.x,
            },
        }
    }
}
