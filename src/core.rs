use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

use crate::data::colour::Rgb;

use crate::data::direction::ALL_DIRECTIONS;
use crate::data::vector2::Vector2;
use crate::{data::grid2d::Grid2D, model::Model};

use crate::entropy_coord::EntropyCoord;
use rand::Rng;
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

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
    #[allow(dead_code)]
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

    pub is_collpased: bool,

    pub tile_enabler_counts: Vec<TileEnablerCount>,
}

unsafe impl Send for CoreCell {}
unsafe impl Sync for CoreCell {}

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

    #[allow(dead_code)]
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

#[derive(Eq, PartialEq)]
pub enum RunStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
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
    pub fn forced_collapse(&mut self, position: Vector2) -> RunStatus {
        // Choose the next lowest cell
        // which hasn't been collapsed yet
        let next_coord = position;

        // Collapse the chosen cell
        let collapse_status = self.collapse_cell_at(next_coord);

        match collapse_status {
            RunStatus::Failed => return RunStatus::Failed,
            RunStatus::Succeeded => {
                self.propagate();
                self.remaining_uncollapsed_cells -= 1;
            }
        }

        // Propagate the effects
        RunStatus::Succeeded
    }

    pub fn collapse_middle(&mut self) -> (CoreState, CoreState, CoreState, CoreState) {
        let sample_size = self.model.samples[0].region.width;
        let middle = self.grid.width / 2;
        let vertical_middle = self.grid.height / 2;

        let x = middle - ((sample_size / 2) + 1);
        let y = vertical_middle - ((sample_size / 2) + 1);

        let mut left_entropy = BinaryHeap::<EntropyCoord>::new();
        let mut left_bottom_entropy = BinaryHeap::<EntropyCoord>::new();
        let mut right_entropy = BinaryHeap::<EntropyCoord>::new();
        let mut right_bottom_entropy = BinaryHeap::<EntropyCoord>::new();

        let mut collapse_target = BinaryHeap::new();

        for pos_y in 0..self.grid.height {
            for pos_x in 0..self.grid.width {
                let in_horizontal = pos_x > x && pos_x < x + sample_size + 1;
                let in_vertical = pos_y > y && pos_y < y + sample_size + 1;

                // Vertical strip
                if in_horizontal || in_vertical {
                    let pos = Vector2 {
                        x: pos_x as i32,
                        y: pos_y as i32,
                    };
                    collapse_target.push(EntropyCoord {
                        coord: pos,
                        entropy: self.grid.get(pos).unwrap().entropy(),
                    });
                }
            }
        }

        while let Some(entropy_coord) = collapse_target.pop() {
            let cell = self.grid.get(entropy_coord.coord).unwrap();

            if cell.is_collpased {
                continue;
            }

            self.forced_collapse(entropy_coord.coord);

            for &direction in &ALL_DIRECTIONS {
                // Propagate the effect to the neighbor in each direction
                let neighbour_coord = entropy_coord.coord.neighbor(direction);

                let in_horizontal = neighbour_coord.x as usize > x
                    && (neighbour_coord.x as usize) < x + sample_size + 1;
                let in_vertical = neighbour_coord.y as usize > y
                    && (neighbour_coord.y as usize) < y + sample_size + 1;

                if self.grid.valid_pos(neighbour_coord) && (in_horizontal || in_vertical) {
                    let neighbor_cell = self.grid.get(neighbour_coord).unwrap();
                    if neighbor_cell.entropy() < entropy_coord.entropy {
                        collapse_target.push(EntropyCoord {
                            coord: neighbour_coord,
                            entropy: neighbor_cell.entropy(),
                        });
                    }
                }
            }
        }

        for pos_y in 0..self.grid.height {
            for pos_x in 0..self.grid.width {
                let in_horizontal = pos_x > x && pos_x < x + sample_size + 1;
                let in_vertical = pos_y > y && pos_y < y + sample_size + 1;

                // Vertical strip
                if in_horizontal || in_vertical {
                    continue;
                }

                // top left
                if pos_x < middle && pos_y < vertical_middle {
                    left_entropy.push(EntropyCoord {
                        entropy: self
                            .grid
                            .get(Vector2 {
                                x: pos_x as i32,
                                y: pos_y as i32,
                            })
                            .unwrap()
                            .entropy(),
                        coord: Vector2 {
                            x: pos_x as i32,
                            y: pos_y as i32,
                        },
                    });
                // bottom left
                } else if pos_x < middle && pos_y > vertical_middle {
                    left_bottom_entropy.push(EntropyCoord {
                        entropy: self
                            .grid
                            .get(Vector2 {
                                x: pos_x as i32,
                                y: pos_y as i32,
                            })
                            .unwrap()
                            .entropy(),
                        coord: Vector2 {
                            x: pos_x as i32,
                            y: (pos_y - vertical_middle) as i32,
                        },
                    });
                // top right
                } else if pos_x >= middle && pos_y < vertical_middle {
                    right_entropy.push(EntropyCoord {
                        entropy: self
                            .grid
                            .get(Vector2 {
                                x: pos_x as i32,
                                y: pos_y as i32,
                            })
                            .unwrap()
                            .entropy(),
                        coord: Vector2 {
                            x: (pos_x - middle) as i32,
                            y: pos_y as i32,
                        },
                    });
                // bottom right
                } else if pos_x >= middle && pos_y > vertical_middle {
                    right_bottom_entropy.push(EntropyCoord {
                        entropy: self
                            .grid
                            .get(Vector2 {
                                x: pos_x as i32,
                                y: pos_y as i32,
                            })
                            .unwrap()
                            .entropy(),
                        coord: Vector2 {
                            x: (pos_x - middle) as i32,
                            y: (pos_y - vertical_middle) as i32,
                        },
                    });
                }
            }
        }

        let make_grid = |o_x, o_y, s_x, s_y| {
            self.grid
                .clone_range(Vector2 { x: o_x, y: o_y }, Vector2 { x: s_x, y: s_y })
        };

        let left_grid = make_grid(0, 0, middle as i32, vertical_middle as i32);
        let left_bottom_grid = make_grid(
            0,
            vertical_middle as i32,
            middle as i32,
            vertical_middle as i32,
        );
        let right_grid = make_grid(middle as i32, 0, middle as i32, vertical_middle as i32);
        let right_bottom_grid = make_grid(
            middle as i32,
            vertical_middle as i32,
            middle as i32,
            vertical_middle as i32,
        );

        let get_remaining =
            |grid: &Grid2D<CoreCell>| grid.data.iter().filter(|cell| !cell.is_collpased).count();

        let left_remains = get_remaining(&left_grid);
        let left_bottom_remains = get_remaining(&left_bottom_grid);
        let right_remains = get_remaining(&right_grid);
        let right_bottom_remains = get_remaining(&right_bottom_grid);

        let make_cs = |grid, remain, entropy| -> CoreState {
            CoreState {
                grid,
                remaining_uncollapsed_cells: remain,
                model: self.model.clone(),
                entropy_heap: entropy,
                tile_removals: VecDeque::new(),
            }
        };

        let left_cs = make_cs(left_grid, left_remains, left_entropy);
        let left_bottom_cs = make_cs(left_bottom_grid, left_bottom_remains, left_bottom_entropy);
        let right_cs = make_cs(right_grid, right_remains, right_entropy);
        let right_bottom_cs = make_cs(
            right_bottom_grid,
            right_bottom_remains,
            right_bottom_entropy,
        );

        (left_cs, right_cs, left_bottom_cs, right_bottom_cs)
    }

    #[allow(dead_code)]
    pub fn is_collpased(&self) -> bool {
        self.remaining_uncollapsed_cells == 0
    }

    pub fn par_process(
        path: &str,
        dimensions: usize,
        width: usize,
        height: usize,
        rotation: bool,
    ) -> Vec<Rgb> {
        println!("Image Processing...");

        let model_creation_time = Instant::now();
        let mut corestate = CoreState::new(path, dimensions, width, height, rotation);
        println!(
            "Model Creation Elapsed Time: {:.2?}",
            model_creation_time.elapsed()
        );

        let grid_res: Vec<_> = {
            let mut res = vec![];

            while res.len() < 4 {
                let model_split = Instant::now();

                corestate = CoreState::new(path, dimensions, width, height, rotation);

                println!("Attempting Model Split...");
                let (left, right, left_bottom, right_bottom) = corestate.collapse_middle();
                println!("Model Split Success... {:.2?}", model_split.elapsed());

                println!();

                res = vec![left, right, left_bottom, right_bottom]
                    .par_iter_mut()
                    .enumerate()
                    .flat_map(|(id, cs)| cs.restart(id as u8))
                    .collect();
            }
            res
        };

        let left = &grid_res[0];
        let right = &grid_res[1];
        let left_bottom = &grid_res[2];
        let right_bottom = &grid_res[3];

        // let left_bottom = left_bottom.restart();

        // println!("Computation Elapsed Time: {:.2?}", compute_time.elapsed());

        // Copy result into output grid

        let mut output_grid = Grid2D::init(width, height, 0);

        for (coord, cell) in left.enumerate() {
            if let Some(tile_index) = cell.get_the_only_possible_tile_index() {
                output_grid.set(coord, tile_index);
            }
        }

        for (coord, cell) in right.enumerate() {
            if let Some(tile_index) = cell.get_the_only_possible_tile_index() {
                output_grid.set(
                    Vector2 {
                        x: coord.x + (width / 2) as i32,
                        y: coord.y,
                    },
                    tile_index,
                );
            }
        }

        for (coord, cell) in left_bottom.enumerate() {
            if let Some(tile_index) = cell.get_the_only_possible_tile_index() {
                output_grid.set(
                    Vector2 {
                        x: coord.x,
                        y: coord.y + (height / 2) as i32,
                    },
                    tile_index,
                );
            }
        }

        for (coord, cell) in right_bottom.enumerate() {
            if let Some(tile_index) = cell.get_the_only_possible_tile_index() {
                output_grid.set(
                    Vector2 {
                        x: coord.x + (width / 2) as i32,
                        y: coord.y + (height / 2) as i32,
                    },
                    tile_index,
                );
            }
        }

        output_grid
            .data
            .iter()
            .map(|&sample_id| corestate.model.samples[sample_id].get_top_left_pixel())
            .collect()
    }

    pub fn restart(&mut self, process_id: u8) -> Option<Grid2D<CoreCell>> {
        let snapshot = self.clone();

        let retry_count = 30;
        let mut count = 0;
        loop {
            count += 1;
            let mut candidates = vec![snapshot.clone(); 4];

            let candidates_result = candidates
                .par_iter_mut()
                .flat_map(|candidate| {
                    let (status, grid) = candidate.run();

                    if status == RunStatus::Succeeded {
                        Ok(grid)
                    } else {
                        Err(())
                    }
                })
                .find_any(|_| true);

            if let Some(candid_res) = candidates_result {
                println!("Subsection Completed: {}", process_id);
                return Some(candid_res.clone());
            }

            if count > retry_count {
                return None;
            }
        }
    }

    pub fn new(
        path: &str,
        dimensions: usize,
        width: usize,
        height: usize,
        rotation: bool,
    ) -> CoreState {
        let model = Model::create(path, dimensions, rotation);
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
        cs.grid.data.par_iter_mut().for_each(|cell: &mut CoreCell| {
            cell.tile_enabler_counts = enabler_counts.clone();
        });

        // Fill the binary heap with the new
        // entropy information after adding noise
        cs.entropy_heap = (0..cs.grid.size())
            .into_par_iter()
            .map(|idx| {
                let coord = cs.grid.to_coord(idx).unwrap();
                let entropy = cs.grid.get(coord).unwrap().entropy();
                EntropyCoord::new(entropy, coord)
            })
            .collect::<BinaryHeap<_>>();

        cs
    }

    //
    // Apply abit of noise to all entropy values
    // to lower the chance of having ties
    //
    fn distribute_entropy_noise(&mut self) {
        self.grid
            .data
            .par_iter_mut()
            .for_each(|cell: &mut CoreCell| {
                let mut rng = rand::thread_rng();
                cell.entropy_noise = rng.gen();
            });
    }

    //
    // Find the next cell which should be collapsed (lowest entropy)
    //
    pub fn choose_next_cell(&mut self) -> Option<Vector2> {
        // Pop the entry with the lowest entropy
        while let Some(entropy_coord) = self.entropy_heap.pop() {
            let cell = self.grid.get(entropy_coord.coord).unwrap();

            // If the cell hasn't been collapsed yet, we take it
            if !cell.is_collpased {
                return Some(entropy_coord.coord);
            }

            // Otherwise we do nothing...
        }

        // Entropy_heap is empty, but there are still uncollapsed cells
        // Just fail and retry
        None
    }

    //
    // Collapse the cell at the given position.
    //
    #[allow(dead_code)]
    fn collapse_cell_at(&mut self, coord: Vector2) -> RunStatus {
        let cell = self.grid.get_mut(coord).unwrap();

        let sample_index_chosen = {
            if let Some(idx) = cell.choose_sample_index(&self.model) {
                idx
            } else {
                return RunStatus::Failed;
            }
        };

        // Set cell to collapsed
        cell.collapsed();

        cell.possible.remove(sample_index_chosen);

        cell.possible.iter().for_each(|tile_index| {
            self.tile_removals
                .push_back(RemovalUpdate { tile_index, coord });
        });

        // Remove ALL other possibilities
        cell.possible.clear();

        // Add the only one posibility
        cell.possible.insert(sample_index_chosen);

        // Note: We don't need to call remove_tile here because
        // we simply don't care about the tile's entropy anymore, there
        // is no point in recalculating it.
        RunStatus::Succeeded
    }

    //
    // Basic search and kill loop
    //
    #[allow(dead_code)]
    fn run(&mut self) -> (RunStatus, &Grid2D<CoreCell>) {
        while self.remaining_uncollapsed_cells > 0 {
            // Choose the next lowest cell
            // which hasn't been collapsed yet

            let next_coord = match self.choose_next_cell() {
                Some(coord) => coord,
                None => {
                    return (RunStatus::Failed, &self.grid);
                }
            };

            // Collapse the chosen cell
            let collapse_status = self.collapse_cell_at(next_coord);

            match collapse_status {
                RunStatus::Failed => {
                    return (
                        RunStatus::Failed,
                        //Grid2D::init(1, 1, CoreCell::new(1, &self.model)),
                        &self.grid,
                    );
                }
                RunStatus::Succeeded => {
                    self.propagate();
                    self.remaining_uncollapsed_cells -= 1;
                }
            }

            // Propagate the effects
        }
        (RunStatus::Succeeded, &self.grid)
    }

    //
    // Remove possibilities based on collapsed cell
    //
    fn propagate(&mut self) {
        while let Some(removal_update) = self.tile_removals.pop_front() {
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
                        if neighbor.is_collpased
                            || neighbor.tile_enabler_counts[compatible_tile]
                                .by_direction
                                .iter()
                                .enumerate()
                                .filter(|(dir, _)| dir != &(direction.opposite().to_idx()))
                                .map(|(_, v)| v)
                                .any(|&v| v == 0)
                        {
                            continue;
                        }

                        self.grid
                            .get_mut(neighbour_coord)
                            .unwrap()
                            .remove_tile(compatible_tile, &self.model);

                        let entropy = EntropyCoord {
                            entropy: self.grid.get(neighbour_coord).unwrap().entropy(),
                            coord: neighbour_coord,
                        };
                        self.entropy_heap.push(entropy);

                        self.tile_removals.push_back(RemovalUpdate {
                            tile_index: compatible_tile,
                            coord: neighbour_coord,
                        });
                    }
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
            let mut cs = CoreState::new("samples/Flowers.png", 3, 10, 10, false);

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
            let mut cs2 = CoreState::new("samples/Flowers.png", 3, 10, 10, false);
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
                0
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
        let mut cs = CoreState::new("samples/Flowers.png", 3, 50, 50, false);

        for _ in 0..cs.grid.size() {
            let least_entropy = &cs.entropy_heap.peek();
            let least_entropy_pos = least_entropy.unwrap().coord;
            assert_eq!(cs.choose_next_cell().unwrap(), least_entropy_pos)
        }
    }

    #[test]
    fn test_basic_collapse() {
        let mut cs = CoreState::new("samples/Flowers.png", 3, 3, 3, false);

        // Check that the same collapsed cell is never visited again
        let mut positions_collapsed = bit_set::BitSet::new();

        // Check that the next entropy is higher than the previous
        let mut last_entropy = -100f32;

        while cs.remaining_uncollapsed_cells > 0 {
            // Find next cell to collapse
            let pos = cs.choose_next_cell().unwrap();

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
        let cs = CoreState::new("samples/Flowers.png", 3, 5, 5, false);

        let init_enablers_count = cs.model.get_initial_tile_enabler_counts();

        cs.grid.data.iter().for_each(|cell: &CoreCell| {
            assert_eq!(&cell.tile_enabler_counts, &init_enablers_count);
        });
    }

    #[test]
    fn test_enablers_count_specific() {
        let cs = CoreState::new("samples/ProcessExample.png", 3, 5, 5, false);

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
        let cs = CoreState::new("samples/ProcessExample.png", 3, 5, 5, false);

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
