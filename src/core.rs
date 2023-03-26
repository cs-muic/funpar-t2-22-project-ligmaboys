use std::collections::{BinaryHeap, VecDeque};

use crate::data::colour::Rgb;
use crate::data::direction::{self, Direction, ALL_DIRECTIONS};
use crate::data::vector2::Vector2;
use crate::{data::grid2d::Grid2D, model::Model};

use crate::entropy_coord::EntropyCoord;
use rand::Rng;

#[allow(dead_code)]
pub type TileIndex = usize;

// Indicate that the potential for tile_index appearing
// in the cell at the coordinate has been removed

#[derive(Debug, Clone)]
pub struct RemovalUpdate {
    tile_index: TileIndex,
    coord: Vector2,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileEnablerCount {
    // `by_direction[d]` will return the count
    // of enablers in the direction 'd'
    pub by_direction: [usize; 4],
}
impl TileEnablerCount {
    pub fn contains_any_zero_count(&self) -> bool {
        self.by_direction.iter().any(|&count| count == 0)
    }
}

// Cell state
#[derive(Debug, Clone)]
pub struct CoreCell {
    // This indiciates all of the possible values in a
    // given cell, initially every value is set to true.
    possible: bit_set::BitSet,

    pub sum_of_possible_tile_weights: u32,

    pub sum_of_possible_tile_weight_log_weights: f32,

    entropy_noise: f32,

    is_collpased: bool,

    pub tile_enabler_counts: Vec<TileEnablerCount>,
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
            tile_enabler_counts: Vec::new(),
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
        // + self.entropy_noise
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
    fn choose_sample_index(&self, context: &Model) -> Option<TileIndex> {
        let mut rng = rand::thread_rng();

        if self.sum_of_possible_tile_weights == 0 {
            return None;
        }

        // Choose a random position in the distribution strip
        let mut remaining = rng.gen_range(0..self.sum_of_possible_tile_weights);

        for possible_sample_indx in &self.possible {
            // This weight represents the width of the section on the strip
            let weight = context.get_relative_freq(possible_sample_indx).0;

            if remaining >= weight {
                remaining -= weight;
            } else {
                return Some(possible_sample_indx);
            }
        }

        // should not end up here
        unreachable!("sum_of_possible_weights was inconsistent with possible_tile_iter and FrequencyHints::relative_frequency");
    }

    fn has_no_possible_tiles(&self) -> bool {
        self.possible.is_empty()
    }

    fn get_the_only_possible_tile_index(&self) -> Option<usize> {
        if self.possible.is_empty() || self.possible.len() > 1 {
            None
        } else {
            self.possible.iter().next()
        }
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

    pub tile_removals: VecDeque<RemovalUpdate>,
}

impl CoreState {
    #[allow(dead_code)]
    pub fn is_collpased(&self) -> bool {
        self.remaining_uncollapsed_cells == 0
    }

    pub fn process(path: &str, dimensions: usize, width: usize, height: usize) -> Vec<Rgb> {
        let mut corestate = CoreState::new(path, dimensions, width, height);

        // dbg!(&corestate.model.freq_map);
        corestate.run();

        // Copy result into output grid

        let mut output_grid = Grid2D::init(width, height, 0);

        for (coord, cell) in corestate.grid.enumerate() {
            if let Some(tile_index) = cell.get_the_only_possible_tile_index() {
                output_grid.set(coord, tile_index);
            }
        }

        output_grid
            .data
            .iter()
            .map(|&sample_id| corestate.model.samples[sample_id].get_top_left_pixel())
            .collect()
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
            tile_removals: VecDeque::new(),
        };

        cs.distribute_entropy_noise();

        let enabler_counts = cs.model.get_initial_tile_enabler_counts();

        // Set the enabler count for all cells
        cs.grid.data.iter_mut().for_each(|cell: &mut CoreCell| {
            cell.tile_enabler_counts = enabler_counts.clone();
        });

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
            cell.entropy_noise = rng.gen_range(0.0f32..0.0000001f32);
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
                println!("COLLAPSED! -> {:?}", &entropy_coord.coord);
                return entropy_coord.coord;
            }

            // Otherwise we do nothing...
        }

        // Remaining cells > 0 but heap is empty...
        dbg!(&self
            .grid
            .data
            .iter()
            .filter(|v| v.possible.len() != 1)
            .count());
        unreachable!("entropy_heap is empty, but there are still uncollapsed cells");
    }

    //
    // Collapse the cell at the given position.
    //
    #[allow(dead_code)]
    fn collapse_cell_at(&mut self, coord: Vector2) {
        // println!("collapsing: cell {:?}", coord);
        // println!("collapsing: cell {}", self.grid.idx(coord).unwrap());

        let cell = self.grid.get_mut(coord).unwrap();

        let sample_index_chosen = cell.choose_sample_index(&self.model).unwrap();

        println!(
            "ENABLERS: {:?}",
            cell.tile_enabler_counts[sample_index_chosen]
        );

        println!(
            "Collapsed to: {:?}",
            &self.model.samples[sample_index_chosen].region.data
        );

        for idx in &self.model.adjacency_rule[sample_index_chosen][Direction::Up.to_idx()] {
            println!("UP CAN BE: {:?}", &self.model.samples[idx].region.data);
        }
        println!(
            "Up size: {}\n",
            self.model.adjacency_rule[sample_index_chosen][Direction::Up.to_idx()].len()
        );

        for idx in &self.model.adjacency_rule[sample_index_chosen][Direction::Right.to_idx()] {
            println!("RIGHT CAN BE: {:?}", &self.model.samples[idx].region.data);
        }
        println!(
            "Right size: {}\n",
            self.model.adjacency_rule[sample_index_chosen][Direction::Right.to_idx()].len()
        );

        for idx in &self.model.adjacency_rule[sample_index_chosen][Direction::Down.to_idx()] {
            println!("BOTTOM CAN BE: {:?}", &self.model.samples[idx].region.data);
        }
        println!(
            "Down size: {}\n",
            self.model.adjacency_rule[sample_index_chosen][Direction::Down.to_idx()].len()
        );

        for idx in &self.model.adjacency_rule[sample_index_chosen][Direction::Left.to_idx()] {
            println!("LEFT CAN BE: {:?}", &self.model.samples[idx].region.data);
        }
        println!(
            "Left size: {}\n",
            self.model.adjacency_rule[sample_index_chosen][Direction::Left.to_idx()].len()
        );

        // Set cell to collapsed
        cell.collapsed();

        cell.possible.remove(sample_index_chosen);

        cell.possible.iter().for_each(|tile_index| {
            self.tile_removals
                .push_back(RemovalUpdate { tile_index, coord });
        });

        println!("INIT REMOVAL LIST: {:?}", &self.tile_removals);

        // Remove ALL other possibilities
        cell.possible.clear();

        // Add the only one posibility
        cell.possible.insert(sample_index_chosen);

        println!("After collapse {:?}", &cell.possible);

        // Note: We don't need to call remove_tile here because
        // we simply don't care about the tile's entropy anymore, there
        // is no point in recalculating it.
    }

    //
    // Basic search and kill loop
    //
    #[allow(dead_code)]
    fn run(&mut self) {
        let mut iter = 0;
        while self.remaining_uncollapsed_cells > 0 {
            // dbg!(iter);

            // Choose the next lowest cell
            // which hasn't been collapsed yet
            let next_coord = self.choose_next_cell();

            // Collapse the chosen cell
            self.collapse_cell_at(next_coord);

            // Propagate the effects
            self.propagate();

            // dbg!("FINISHED PROPAGATION");

            self.remaining_uncollapsed_cells -= 1;

            iter += 1;
        }
    }

    //
    // Remove possibilities based on collapsed cell
    //
    fn propagate(&mut self) {
        while let Some(removal_update) = self.tile_removals.pop_front() {
            println!("====================================================================================================");
            println!();
            println!("{:?}", &removal_update);
            println!();
            for y in 0..self.grid.height as i32 {
                for x in 0..self.grid.width as i32 {
                    let possible = self.grid.get(Vector2 { x, y }).unwrap().possible.clone();
                    let enablers: Vec<_> = possible
                        .iter()
                        .map(|idx| {
                            self.grid.get(Vector2 { x, y }).unwrap().tile_enabler_counts[idx]
                                .by_direction
                        })
                        .collect();

                    let zipped = possible.iter().zip(enablers.iter()).collect::<Vec<_>>();

                    print!(
                        "{}",
                        format!("{:width$}", format!("({:?})", zipped), width = 50)
                    );
                }
                println!();
            }

            'dir: for &direction in &ALL_DIRECTIONS {
                // Propagate the effect to the neighbor in each direction
                let neighbour_coord = removal_update.coord.neighbor(direction);

                if let Some(cell_to_update) = self.grid.get(neighbour_coord) {
                    cell_to_update
                } else {
                    continue 'dir;
                };

                // Iterate over all the tiles which may appear in the neighbouring cell
                // (in the direction of the current one)
                for compatible_tile in
                    self.model.adjacency_rule[removal_update.tile_index][direction.to_idx()].iter()
                {
                    let neighbor = self.grid.get_mut(neighbour_coord).unwrap();

                    let count = {
                        let count = &mut neighbor.tile_enabler_counts[compatible_tile].by_direction
                            [direction.opposite().to_idx()];
                        if *count == 0 {
                            continue;
                        }
                        *count -= 1;
                        *count
                    };

                    // If count is 0, we want to remove the tile from the neighbour
                    if count == 0 {
                        
                        self.grid
                            .get_mut(neighbour_coord)
                            .unwrap()
                            .remove_tile(compatible_tile, &self.model);

                        self.entropy_heap.push(EntropyCoord {
                            entropy: self.grid.get(neighbour_coord).unwrap().entropy(),
                            coord: neighbour_coord,
                        });

                        self.tile_removals.push_back(RemovalUpdate {
                            tile_index: compatible_tile,
                            coord: neighbour_coord,
                        });

                    }

                    // let opposite_direction = direction.opposite().to_idx();

                    // // Look up the count of enablers
                    // let enabler_counts = &mut neighbour_cell.tile_enabler_counts[compatible_tile];

                    // if enabler_counts.by_direction[opposite_direction] == 1 {

                    //     // Zero count in another direction means that
                    //     // the tile has already been removed, we want
                    //     // to avoid removing again.
                    //     if !enabler_counts.contains_any_zero_count() {

                    //         neighbour_cell.remove_tile(compatible_tile, &self.model);

                    //         println!("Tile: {:?} has been removed from {:?}", compatible_tile, neighbour_coord);

                    //         if neighbour_cell.has_no_possible_tiles() {

                    //             panic!("Contradiction");
                    //         }

                    //         // dbg!(neighbour_cell.entropy());
                    //         // println!("Lowest -> {:?}", self.entropy_heap.peek());

                    //         self.entropy_heap.push(EntropyCoord {
                    //             entropy: neighbour_cell.entropy(),
                    //             coord: neighbour_coord,
                    //         });

                    //         // Add the update to the stack
                    //         self.tile_removals.push(RemovalUpdate {
                    //             tile_index: compatible_tile,
                    //             coord: neighbour_coord,
                    //         });
                    //     }
                    // }

                    // let enabler_counts = &mut neighbour_cell.tile_enabler_counts[compatible_tile];

                    // if enabler_counts.by_direction[opposite_direction] == 0 {
                    //     continue;
                    // }

                    // enabler_counts.by_direction[opposite_direction] -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{data::direction::Direction, model::Model};

    use super::{CoreCell, CoreState};
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

    #[test]
    fn test_enablers_count() {
        let cs = CoreState::new("samples/Flowers.png", 3, 5, 5);

        let init_enablers_count = cs.model.get_initial_tile_enabler_counts();

        cs.grid.data.iter().for_each(|cell: &CoreCell| {
            assert_eq!(&cell.tile_enabler_counts, &init_enablers_count);
        });
    }

    #[test]
    fn test_enablers_count_specific() {
        let cs = CoreState::new("samples/ProcessExample.png", 3, 5, 5);

        let sample_1 = find_sample_idx(
            &cs.model,
            vec![
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
            ],
        )
        .unwrap();

        let init_enablers_count = cs.model.get_initial_tile_enabler_counts();
        let sample_1_enablers_count = &init_enablers_count[sample_1];

        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Up.to_idx()],
            1
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Right.to_idx()],
            2
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Left.to_idx()],
            1
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Down.to_idx()],
            2
        );
    }

    #[test]
    fn test_enablers_count_specific_2() {
        let cs = CoreState::new("samples/ProcessExample.png", 3, 5, 5);

        let sample_1 = find_sample_idx(
            &cs.model,
            vec![
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ],
        )
        .unwrap();

        let init_enablers_count = cs.model.get_initial_tile_enabler_counts();
        let sample_1_enablers_count = &init_enablers_count[sample_1];

        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Up.to_idx()],
            2
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Right.to_idx()],
            2
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Left.to_idx()],
            2
        );
        assert_eq!(
            sample_1_enablers_count.by_direction[Direction::Down.to_idx()],
            2
        );

        assert!(cs.model.adjacency_rule[sample_1][Direction::Down.to_idx()].contains(sample_1));
    }
}
