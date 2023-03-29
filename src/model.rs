use std::collections::HashMap;

use crate::core::TileEnablerCount;
use crate::data::direction::{Direction, ALL_DIRECTIONS};
use crate::data::sample::{Sample, SampleID};

extern crate bit_set;
extern crate image;
use crate::image_reader;

#[derive(Debug)]
pub struct Model {
    pub samples: Vec<Sample>,
    pub freq_map: Vec<(SampleID, (u32, f32))>,
    pub adjacency_rule: Vec<[bit_set::BitSet; 4]>,
}

impl Model {
    pub fn get_relative_freq(&self, sample_id: SampleID) -> (u32, f32) {
        self.freq_map[sample_id].1
    }

    pub fn create(img_path: &str, n_dimensions: usize) -> Model {
        // Parse CLI <ImgPath> <Shape>

        // Load image from args passed in
        let img: image::DynamicImage = image::open(img_path).expect("Failed to open image");

        // Process image
        let mut image = image_reader::Image::new(img.width() as usize, img.height() as usize);
        image.load(&img);

        // Retrieve image samples (includes duplicates)
        let mut unprocessed_samples = image.sample(n_dimensions as i32);
        unprocessed_samples = unprocessed_samples
            .iter()
            .flat_map(|sample| vec![sample.clone(), sample.rev_sample()])
            .collect();

        // Calculate the number of times each unique sample appears
        let freq_map: HashMap<Sample, i32> = unprocessed_samples.iter().fold(
            HashMap::<Sample, i32>::new(),
            |mut freq_map, sample| {
                *freq_map.entry(sample.clone()).or_insert(0) += 1;
                freq_map
            },
        );

        // Unzip the frequency map
        let (samples, freqs): (Vec<_>, Vec<_>) = freq_map.into_iter().unzip();

        // Assign each frequency to an ID
        // Note: The ID works w.r.t the sample vector
        let freq_mapping: Vec<(SampleID, _)> = freqs
            .iter()
            .enumerate()
            .map(|(i, freq)| {
                (
                    i as SampleID,
                    (*freq as u32, (*freq as f32) * (*freq as f32).log2()),
                )
            })
            .collect();

        // In the form [s1][direction][s2]
        let sample_size = samples.len();
        let bitsets: [bit_set::BitSet; 4] = [
            bit_set::BitSet::with_capacity(sample_size),
            bit_set::BitSet::with_capacity(sample_size),
            bit_set::BitSet::with_capacity(sample_size),
            bit_set::BitSet::with_capacity(sample_size),
        ];

        let mut adjacency_rules: Vec<[bit_set::BitSet; 4]> = vec![bitsets; samples.len()];

        // Create adjacency rules
        for s1 in 0..samples.len() {
            for s2 in 0..samples.len() {
                for direction in &ALL_DIRECTIONS {
                    if samples[s1].compatible(&samples[s2], *direction) {
                        adjacency_rules[s1][direction.to_idx()].insert(s2);
                    }
                }
            }
        }

        Model {
            samples,
            freq_map: freq_mapping,
            adjacency_rule: adjacency_rules,
        }
    }

    pub fn get_initial_tile_enabler_counts(&self) -> Vec<TileEnablerCount> {
        let mut ret: Vec<TileEnablerCount> = Vec::new();

        for tile_a in 0..self.samples.len() {
            let mut counts = TileEnablerCount {
                by_direction: [0, 0, 0, 0],
            };

            for &direction in &ALL_DIRECTIONS {
                let dir = direction.to_idx();
                counts.by_direction[dir] = self.adjacency_rule[tile_a][dir].len();
            }

            ret.push(counts);
        }
        ret
    }

    pub fn size(&self) -> usize {
        self.samples.len()
    }

    #[allow(dead_code)]
    pub fn get_possible_nbrs(&self, sample_idx: SampleID, dir: Direction) -> Option<Vec<SampleID>> {
        let nbrs = &self.adjacency_rule[sample_idx][dir.to_idx()];
        let nbrs: Vec<SampleID> = nbrs.iter().collect();

        if nbrs.is_empty() {
            None
        } else {
            Some(nbrs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::data::grid2d::Grid2D;

    fn find_sample_idx(model: &Model, sample: Vec<[u8; 3]>) -> Option<usize> {
        model
            .samples
            .clone()
            .iter()
            .position(|v| v.region.data == sample)
    }

    #[test]
    fn check_valid_model() {
        use super::*;
        use crate::data::direction::Direction;
        let model = Model::create("samples/ProcessExampleLong.png", 3);
        assert!(model.size() == 16);

        let sample_1 = find_sample_idx(
            &model,
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ],
        )
        .unwrap();

        // Find the bottom compatible tile
        let compatible: Vec<_> = model.get_possible_nbrs(sample_1, Direction::Down).unwrap();
        let bottom_compat = &model.samples[*&compatible[0]];
        let picked_sample = &model.samples[sample_1];

        assert_eq!(
            bottom_compat.region.data.clone(),
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0]
            ]
        );

        assert_eq!(
            picked_sample.region.data.clone(),
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
        );

        // Should return origin
        let compatible: Vec<_> = model
            .get_possible_nbrs(*&compatible[0], Direction::Up)
            .unwrap();
        let left_compat = &model.samples[*&compatible[0]];
        assert_eq!(
            left_compat.region.data.clone(),
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
        );

        // Find the right compatible tile
        let compatible: Vec<_> = model.get_possible_nbrs(sample_1, Direction::Right).unwrap();
        let right_compat = &model.samples[*&compatible[0]];
        assert_eq!(
            right_compat.region.data.clone(),
            vec![
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
        );

        // Should return origin
        let compatible: Vec<_> = model
            .get_possible_nbrs(*&compatible[0], Direction::Left)
            .unwrap();
        let left_compat = &model.samples[*&compatible[0]];
        assert_eq!(
            left_compat.region.data.clone(),
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [136, 136, 255],
                [136, 136, 255],
                [136, 136, 255],
            ]
        );

        let sample_1 = find_sample_idx(
            &model,
            vec![
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
                [0, 0, 0],
                [136, 136, 255],
                [0, 0, 0],
            ],
        )
        .unwrap();

        let compatible_bottom: Vec<_> = model.get_possible_nbrs(sample_1, Direction::Down).unwrap();
        let compatible_top: Vec<_> = model.get_possible_nbrs(sample_1, Direction::Up).unwrap();
        // let left_compat = &model.samples[*&compatible[0]];
        // assert_eq!(
        //     left_compat.region.clone(),
        //     vec![
        //         [0, 0, 0],
        //         [136, 136, 255],
        //         [0, 0, 0],
        //         [0, 0, 0],
        //         [136, 136, 255],
        //         [0, 0, 0],
        //         [136, 136, 255],
        //         [136, 136, 255],
        //         [136, 136, 255],
        //     ]
        // );

        let samples = compatible_top
            .iter()
            .copied()
            .map(|sample_id| &model.samples[sample_id as usize])
            .collect::<Vec<_>>();

        assert!(&samples.contains(&&Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: vec![
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                ],
            }
        }));

        let samples = compatible_bottom
            .iter()
            .copied()
            .map(|sample_id| &model.samples[sample_id as usize])
            .collect::<Vec<_>>();

        assert!(&samples.contains(&&Sample {
            region: Grid2D {
                width: 3,
                height: 3,
                data: vec![
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                    [0, 0, 0],
                    [136, 136, 255],
                    [0, 0, 0],
                ],
            }
        }));
    }
}
