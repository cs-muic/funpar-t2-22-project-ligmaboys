use crate::{data::grid2d::Grid2D, model::Model};

type TileIndex = usize;

// Cell state
struct CoreCell {
    // This indiciates all of the possible values in a
    // given cell, initially every value is set to true.
    possible: bit_set::BitSet,
}

struct CoreState {
    // Output grid
    grid: Grid2D<CoreCell>,

    // Number of cells that hasn't been
    // collapsed yet, intialized to grid.len()
    remaining_uncollapsed_cells: usize,

    // Our wfc model, contains the rules
    // we need in order to collapse tiles.
    model: Model,
}

impl CoreState {
    #[allow(dead_code)]
    pub fn is_collpased(&self) -> bool {
        self.remaining_uncollapsed_cells == 0
    }
}
