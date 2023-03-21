use super::vector2::Vector2;

// Note: The attributes are public
//
#[allow(dead_code)]
#[derive(Debug)]
pub struct Grid2D<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Grid2D<T> {
    // Given a Vector2 position, return the index
    // which corresponds to it in the 1D collection.
    
    pub fn idx(&self, pos: Vector2) -> Option<usize> {
        if self.valid_pos(pos) {
            todo!()
        } else {
            None
        }
    }

    // BORROW
    // Given a Vector2 position, return the
    // element at the corresponding position.
    #[allow(dead_code)]
    pub fn get(&self, pos: Vector2) -> Option<&T> {
        !todo!()
    }

    // MUTABLE BORROW
    // Given a Vector2 position, return the
    // element at the corresponding position.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, pos: Vector2) -> Option<&mut T> {
        !todo!()
    }

    // Ensure that the position is valid (not out of bounds)
    #[allow(dead_code)]
    pub fn valid_pos(&self, pos: Vector2) -> bool {
        !todo!()
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn some_test() {}
}
