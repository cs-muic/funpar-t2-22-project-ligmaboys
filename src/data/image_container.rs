use super::{colour::Rgb, vector2::Vector2};

pub trait ImageContainer {
    fn idx(&self, at: Vector2) -> usize;
    fn at(&self, at: Vector2) -> Rgb;
}
