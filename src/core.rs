use crate::{
    data::grid2d::Grid2D,
    model::{self, Model},
};

type TileIndex = usize;

// Cell state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CoreCell {
    // This indiciates all of the possible values in a
    // given cell, initially every value is set to true.
    possible: bit_set::BitSet,
}

impl CoreCell {
    pub fn new(capacity: usize) -> CoreCell {
        let mut bs = bit_set::BitSet::with_capacity(capacity);
        (0..capacity).into_iter().for_each(|x| {
            bs.insert(x);
        });
        CoreCell { possible: bs }
    }
    pub fn total_possible_tile_freq(&self, model: &Model) -> u32 {
        self.possible
            .iter()
            .map(|id| model.get_relative_freq(id) as u32)
            .sum()
    }
    pub fn entropy(&self, model: &Model) -> f32 {
        let total_weight = self.total_possible_tile_freq(model) as f32;
        let sum_of_weight_log_weight = self.possible.iter().fold(0f32, |a, sample_id| {
            let rf: f32 = model.get_relative_freq(sample_id);
            //println!("ref: {}", rf);
            a + (rf * rf.log2())
        });
        //println!("{} {}", total_weight, sum_of_weight_log_weight);
        total_weight.log2() - (sum_of_weight_log_weight / total_weight)
    }
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
    pub fn new(path: &str, dimensions: usize, width: usize, height: usize) -> CoreState {
        let model = Model::create(path, dimensions);
        let grid = Grid2D::init(width, height, CoreCell::new(model.samples.len()));
        let remaining_uncollapsed_cells = grid.size();
        CoreState {
            grid,
            remaining_uncollapsed_cells,
            model,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;

    use super::CoreState;
    fn find_sample_idx(model: &Model, sample: Vec<[u8; 3]>) -> Option<usize> {
        model
            .samples
            .clone()
            .iter()
            .position(|v| v.region.data == sample)
    }

    #[test]
    fn test_entropy() {
        let s1 = [
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
        ]
        .to_vec();
        let mut cs = CoreState::new("samples/Flowers.png", 3, 10, 10);
        cs.grid.data[0].possible.clear();
        cs.grid.data[0].possible.insert(1);
        cs.grid.data[0].possible.insert(0);
        println!(
            "{:?}",
            cs.grid
                .data
                .iter()
                .map(|cell| cell.entropy(&cs.model))
                .collect::<Vec<_>>()
        )
    }
}
