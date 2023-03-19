use std::collections::HashMap;

use crate::data::direction::ALL_DIRECTIONS;
use crate::data::sample::{Sample, SampleID};

extern crate image;
use crate::image_reader;

pub struct Model {
    pub samples: Vec<Sample>,
    pub freq_map: Vec<(SampleID, u32)>,
    pub adjacency_rule: Vec<Vec<Vec<bool>>>,
}

impl Model {
    pub fn create(img_path: &str, n_dimensions: usize) -> Model {
        // Parse CLI <ImgPath> <Shape>

        // Load image from args passed in
        let img: image::DynamicImage = image::open(img_path).expect("Failed to open image");

        // Process image
        let mut image = image_reader::Image::new(img.width() as usize, img.height() as usize);
        image.load(&img);

        // Retrieve image samples (includes duplicates)
        let unprocessed_samples = image.sample(n_dimensions as i32);

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
            .map(|(i, freq)| (i as SampleID, *freq as u32))
            .collect();

        // In the form [s1][direction][s2]
        let mut adjacency_rules = vec![vec![vec![false; samples.len()]; 4]; samples.len()];

        // Create adjacency rules
        for s1 in 0..samples.len() {
            for s2 in 0..samples.len() {
                for direction in &ALL_DIRECTIONS {
                    if samples[s1].compatible(&samples[s2], *direction) {
                        adjacency_rules[s1][direction.to_idx()][s2] = true;
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

    pub fn size(&self) -> usize {
        self.samples.len()
    }
}
