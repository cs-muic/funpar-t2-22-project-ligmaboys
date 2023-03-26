use super::vector2::Vector2;
use std::fmt::{self, Debug};

// Note: The attributes are public
//
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grid2D<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T: Clone> Grid2D<T> {
    pub fn init(width: usize, height: usize, init_val: T) -> Grid2D<T> {
        let new_data = vec![init_val; width * height];
        Grid2D {
            width,
            height,
            data: new_data,
        }
    }

    pub fn set(&mut self, pos: Vector2, item: T) {
        let idx = self.idx(pos).unwrap();
        self.data[idx] = item;
    }

    // Given an index, return the coord
    // which corresponds to it in the 2D representation.

    pub fn to_coord(&self, pos: usize) -> Option<Vector2> {
        let y = (pos / self.width) as i32;
        let x = (pos % self.width) as i32;
        let pos = Vector2 { x, y };

        if self.valid_pos(pos) {
            Some(pos)
        } else {
            None
        }
    }

    // Given a Vector2 position, return the index
    // which corresponds to it in the 1D collection.

    pub fn idx(&self, pos: Vector2) -> Option<usize> {
        if self.valid_pos(pos) {
            Some(pos.y as usize * self.width + pos.x as usize)
        } else {
            None
        }
    }

    // BORROW
    // Given a Vector2 position, return the
    // element at the corresponding position.
    #[allow(dead_code)]
    pub fn get(&self, pos: Vector2) -> Option<&T> {
        match self.idx(pos) {
            Some(index) => Some(&self.data[index]),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_copy(&self, pos: Vector2) -> Option<T> {
        match self.idx(pos) {
            Some(index) => Some(self.data[index].clone()),
            None => None,
        }
    }

    // MUTABLE BORROW
    // Given a Vector2 position, return the
    // element at the corresponding position.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, pos: Vector2) -> Option<&mut T> {
        match self.idx(pos) {
            Some(index) => Some(&mut self.data[index]),
            None => None,
        }
    }

    

    // Ensure that the position is valid (not out of bounds)
    #[allow(dead_code)]
    pub fn valid_pos(&self, pos: Vector2) -> bool {
        (0..self.width).contains(&(pos.x as usize)) && (0..self.height).contains(&(pos.y as usize))
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn enumerate(&self) -> impl Iterator<Item=(Vector2, &T)> + '_  {
        self.data.iter().enumerate().map(|(idx, t)| (self.to_coord(idx).unwrap(), t))
    }


}

impl<T: Clone + Debug> Grid2D<T> {
    pub fn print(&self) {
        
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                print!("({:?})", self.get(Vector2 { y, x }));
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn some_test() {}
}
