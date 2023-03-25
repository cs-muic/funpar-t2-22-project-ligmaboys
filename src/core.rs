use std::collections::BinaryHeap;

use crate::data::vector2::Vector2;
use crate::{data::grid2d::Grid2D, model::Model};

use crate::entropy_coord::EntropyCoord;
use rand::Rng;

#[allow(dead_code)]
type TileIndex = usize;

// Cell state
#[derive(Debug, Clone)]
pub struct CoreCell {
    // This indiciates all of the possible values in a
    // given cell, initially every value is set to true.
    possible: bit_set::BitSet,

    sum_of_possible_tile_weights: u32,

    sum_of_possible_tile_weight_log_weights: f32,

    entropy_noise: f32,

    is_collpased: bool,
}

impl CoreCell {
    //
    // Create a new core cell with all possible tiles set to true.
    // (Initializes in a super-position state)
    //
    pub fn new(capacity: usize, context: &Model) -> CoreCell {
        let mut bs = bit_set::BitSet::with_capacity(capacity);
        (0..capacity).for_each(|x| {
            bs.insert(x);
        });

        let mut cell = CoreCell {
            possible: bs,
            sum_of_possible_tile_weights: 0,
            sum_of_possible_tile_weight_log_weights: 0f32,
            entropy_noise: 0f32,
            is_collpased: false,
        };

        cell.sum_of_possible_tile_weights = cell.total_possible_tile_freq(context);

        let sum_of_weight_log_weight = cell.possible.iter().fold(0f32, |a, sample_id| {
            a + context.get_relative_freq(sample_id).1
        });

        cell.sum_of_possible_tile_weight_log_weights = sum_of_weight_log_weight;

        cell
    }

    //
    // Get the sum of all current possible tile's frequency
    //
    pub fn total_possible_tile_freq(&self, model: &Model) -> u32 {
        self.possible
            .iter()
            .map(|id| model.get_relative_freq(id).0)
            .sum()
    }

    pub fn collapsed(&mut self) {
        self.is_collpased = true;
    }

    #[allow(dead_code)]
    pub fn entropy_no_cache(&self, model: &Model) -> f32 {
        let total_weight = self.total_possible_tile_freq(model) as f32;
        let sum_of_weight_log_weight = self.possible.iter().fold(0f32, |a, sample_id| {
            a + model.get_relative_freq(sample_id).1
        });

        total_weight.log2() - (sum_of_weight_log_weight / total_weight)
    }

    //
    // Calculate a cell's entropy (Cached)
    //
    pub fn entropy(&self) -> f32 {
        (self.sum_of_possible_tile_weights as f32).log2()
            - (self.sum_of_possible_tile_weight_log_weights
                / self.sum_of_possible_tile_weights as f32)
            + self.entropy_noise
    }

    //
    //  Given a TileIndex, remove the tile from the cell,
    //  then update the new entropy for the given cell.
    //
    #[allow(dead_code)]
    pub fn remove_tile(&mut self, tile_index: TileIndex, model: &Model) {
        // Remove the tile
        self.possible.remove(tile_index);

        // Recalculate the entropy
        let freq = model.get_relative_freq(tile_index);
        self.sum_of_possible_tile_weights -= freq.0;
        self.sum_of_possible_tile_weight_log_weights -= freq.1;
    }

    //
    // Roulette wheel selection algorithm,
    // Choose a random sample with frequency hints taken into account
    //
    #[allow(dead_code)]
    fn choose_sample_index(&self, context: &Model) -> TileIndex {
        let mut rng = rand::thread_rng();

        // Choose a random position in the distribution strip
        let mut remaining = rng.gen_range(0..self.sum_of_possible_tile_weights);

        for possible_sample_indx in &self.possible {
            // This weight represents the width of the section on the strip
            let weight = context.get_relative_freq(possible_sample_indx).0;

            if remaining >= weight {
                remaining -= weight;
            } else {
                return possible_sample_indx;
            }
        }

        // should not end up here
        unreachable!("sum_of_possible_weights was inconsistent with possible_tile_iter and FrequencyHints::relative_frequency");
    }
}

#[derive(Debug)]
pub struct CoreState {
    // Output grid
    pub grid: Grid2D<CoreCell>,

    // Number of cells that hasn't been
    // collapsed yet, intialized to grid.len()
    pub remaining_uncollapsed_cells: usize,

    // Our wfc model, contains the rules
    // we need in order to collapse tiles.
    pub model: Model,

    pub entropy_heap: BinaryHeap<EntropyCoord>,
}

impl CoreState {
    #[allow(dead_code)]
    pub fn is_collpased(&self) -> bool {
        self.remaining_uncollapsed_cells == 0
    }
    pub fn new(path: &str, dimensions: usize, width: usize, height: usize) -> CoreState {
        let model = Model::create(path, dimensions);
        let grid = Grid2D::init(width, height, CoreCell::new(model.size(), &model));
        let remaining_uncollapsed_cells = grid.size();

        let mut cs = CoreState {
            grid,
            remaining_uncollapsed_cells,
            model,
            entropy_heap: BinaryHeap::new(),
        };

        cs.distribute_entropy_noise();

        // Fill the binary heap with the new
        // entropy information after adding noise
        (0..cs.grid.size()).for_each(|idx| {
            let coord = cs.grid.to_coord(idx).unwrap();
            let entropy = cs.grid.get(coord).unwrap().entropy();
            cs.entropy_heap.push(EntropyCoord::new(entropy, coord))
        });

        cs
    }

    //
    // Apply abit of noise to all entropy values
    // to lower the chance of having ties
    //
    fn distribute_entropy_noise(&mut self) {
        let mut rng = rand::thread_rng();
        self.grid.data.iter_mut().for_each(|cell: &mut CoreCell| {
            cell.entropy_noise = rng.gen_range(-0.0025f32..0.0025f32);
        });
    }

    //
    // Find the next cell which should be collapsed (lowest entropy)
    //
    pub fn choose_next_cell(&mut self) -> Vector2 {
        // Pop the entry with the lowest entropy
        while let Some(entropy_coord) = self.entropy_heap.pop() {
            let cell = self.grid.get(entropy_coord.coord).unwrap();

            // If the cell hasn't been collapsed yet, we take it
            if !cell.is_collpased {
                return entropy_coord.coord;
            }

            // Otherwise we do nothing...
        }

        // Remaining cells > 0 but heap is empty...
        unreachable!("entropy_heap is empty, but there are still uncollapsed cells");
    }

    //
    // Collapse the cell at the given position.
    //
    #[allow(dead_code)]
    fn collapse_cell_at(&mut self, coord: Vector2) {
        let cell = self.grid.get_mut(coord).unwrap();
        let sample_index_chosen = cell.choose_sample_index(&self.model);

        // Set cell to collapsed
        cell.collapsed();

        // Remove ALL other possibilities
        cell.possible.clear();

        // Add the only one posibility
        cell.possible.insert(sample_index_chosen);

        // Note: We don't need to call remove_tile here because
        // we simply don't care about the tile's entropy anymore, there
        // is no point in recalculating it.
    }

    //
    // Basic search and kill loop
    //
    #[allow(dead_code)]
    fn run(&mut self) {
        while self.remaining_uncollapsed_cells > 0 {
            // Choose the next lowest cell
            // which hasn't been collapsed yet
            let next_coord = self.choose_next_cell();

            // Collapse the chosen cell
            self.collapse_cell_at(next_coord);

            // Propagate the effects
            // self.propagate();

            self.remaining_uncollapsed_cells -= 1;
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

    // https://stackoverflow.com/questions/41447678/comparison-of-two-floats-in-rust-to-arbitrary-level-of-precision
    fn approx_equal(a: f64, b: f64, decimal_places: u8) -> bool {
        let factor = 10.0f64.powi(decimal_places as i32);
        let a = (a * factor).trunc();
        let b = (b * factor).trunc();
        a == b
    }

    #[test]
    fn test_removal_entropy() {
        for _ in 0..10 {
            let mut cs = CoreState::new("samples/Flowers.png", 3, 10, 10);

            // For Sample ID
            let target_sample = &cs.model.samples[0];

            cs.grid.data[0].possible.clear();
            cs.grid.data[0].possible.insert(0);

            let non_cached_entropy = cs
                .grid
                .data
                .iter()
                .map(|cell| cell.entropy_no_cache(&cs.model))
                .collect::<Vec<_>>();

            // Cached Version
            let mut cs2 = CoreState::new("samples/Flowers.png", 3, 10, 10);
            let sample_id = find_sample_idx(&cs2.model, target_sample.region.data.clone()).unwrap();

            assert_eq!(
                &cs2.model.samples[sample_id].region.data,
                &target_sample.region.data
            );

            (0..cs2.model.size()).for_each(|idx| {
                if idx == sample_id {
                    return;
                } else {
                    cs2.grid.data[0].remove_tile(idx, &cs2.model)
                }
            });

            let cached_entropy = cs2
                .grid
                .data
                .iter()
                .map(|cell| cell.entropy())
                .collect::<Vec<_>>();

            // Artifact collected from precision error
            assert!(approx_equal(
                non_cached_entropy[0] as f64,
                cached_entropy[0] as f64,
                1
            ))
        }
    }

    //
    // Note: This is only for checking that the next chosen cell is the smallest value
    // (This doesn't take into account cells which have been pushed multiple times into
    //  the binary heap due to recalculation of their entropy, if this test is used in
    //  conjunction with collpase, it will definitely fail.)
    //
    #[test]
    fn test_binary_heap() {
        let mut cs = CoreState::new("samples/Flowers.png", 3, 50, 50);

        for _ in 0..cs.grid.size() {
            let least_entropy = &cs.entropy_heap.peek();
            let least_entropy_pos = least_entropy.unwrap().coord;
            assert_eq!(cs.choose_next_cell(), least_entropy_pos)
        }
    }

    #[test]
    fn test_basic_collapse() {
        let mut cs = CoreState::new("samples/Flowers.png", 3, 3, 3);

        // Check that the same collapsed cell is never visited again
        let mut positions_collapsed = bit_set::BitSet::new();

        // Check that the next entropy is higher than the previous
        let mut last_entropy = -100f32;

        while cs.remaining_uncollapsed_cells > 0 {
            // Find next cell to collapse
            let pos = cs.choose_next_cell();

            // Check if we've visited this same position before
            let grid_idx = cs.grid.idx(pos).unwrap();
            assert!(!positions_collapsed.contains(grid_idx));
            positions_collapsed.insert(grid_idx);

            // Check that after collapsing we only
            // have one posibility left
            cs.collapse_cell_at(pos);
            let cell = cs.grid.get(pos).unwrap();
            assert_eq!(cell.possible.len(), 1);

            // Note: Here we use the value that was
            // cached inside the cell since the beginning,
            // since collapse_cell_at does not recalculate
            // the entropy of the cell
            let new_entropy = cell.entropy();
            assert!(new_entropy > last_entropy);
            last_entropy = new_entropy;

            cs.remaining_uncollapsed_cells -= 1;
        }
    }
}
