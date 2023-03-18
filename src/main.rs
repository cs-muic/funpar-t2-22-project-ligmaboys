use std::collections::HashMap;

use clap::Parser;
use cli::Args;

use crate::data::direction::ALL_DIRECTIONS;
use crate::data::sample::{Sample, SampleID};

extern crate image;

mod cli;
mod data;
mod image_reader;

// TODO: Break down the logic into functions.

fn main() {
    // Parse CLI <ImgPath> <Shape>
    let args: Args = Args::parse();

    // Load image from args passed in
    let img: image::DynamicImage = image::open(args.img_path).expect("Failed to open image");

    // Process image
    let mut image = image_reader::Image::new(img.width() as usize, img.height() as usize);
    image.load(&img);

    // Retrieve image samples (includes duplicates)
    let unprocessed_samples = image.sample(args.n_dimensions as i32);

    // Calculate the number of times each unique sample appears
    let freq_map: HashMap<Sample, i32> =
        unprocessed_samples
            .iter()
            .fold(HashMap::<Sample, i32>::new(), |mut freq_map, sample| {
                *freq_map.entry(sample.clone()).or_insert(0) += 1;
                freq_map
            });

    // Unzip the frequency map
    let (samples, freqs): (Vec<_>, Vec<_>) = freq_map.into_iter().unzip();

    // Assign each frequency to an ID
    // Note: The ID works w.r.t the sample vector
    let _freq_mapping: Vec<(SampleID, _)> = freqs
        .iter()
        .enumerate()
        .map(|(i, freq)| (i as SampleID, freq))
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

    // let sample_1 = samples
    //     .clone()
    //     .iter()
    //     .position(|v| {
    //         v.region
    //             == vec![
    //                 [0, 0, 0],
    //                 [136, 136, 255],
    //                 [0, 0, 0],
    //                 [0, 0, 0],
    //                 [136, 136, 255],
    //                 [0, 0, 0],
    //                 [136, 136, 255],
    //                 [136, 136, 255],
    //                 [136, 136, 255],
    //             ]
    //     })
    //     .unwrap();

    // let compatible: Vec<_> = (0..samples.len())
    //     .into_iter()
    //     .filter(|s_idx| *&adjacency_rules[sample_1][Direction::Down.to_idx()][*s_idx])
    //     .collect();
    // // dbg!(&compatible);
    // dbg!(&samples[*&compatible[0]]);
    // dbg!(&samples[sample_1]);
}
