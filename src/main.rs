use std::collections::HashMap;

use clap::Parser;
use cli::Args;

use crate::data::sample::{Sample, SampleID};

extern crate image;

mod cli;
mod data;
mod image_reader;

fn main() {
    // Parse CLI <ImgPath> <Shape>
    let args: Args = Args::parse();

    // Load image from args passed in
    let img: image::DynamicImage = image::open(args.img_path).expect("Failed to open image");

    // Process image
    let mut image = image_reader::Image::new(img.width() as usize, img.height() as usize);
    image.load(&img);

    // Retrieve image samples (includes duplicates)
    let samples = image.sample(args.n_dimensions as i32);

    dbg!(&samples);

    // Calculate the number of times each unique sample appears
    let freq_map: HashMap<Sample, i32> =
        samples
            .iter()
            .fold(HashMap::<Sample, i32>::new(), |mut freq_map, sample| {
                *freq_map.entry(sample.clone()).or_insert(0) += 1;
                freq_map
            });

    // Unzip the frequency map
    let (_samples, freqs): (Vec<_>, Vec<_>) = freq_map.into_iter().unzip();

    // Assign each frequency to an ID
    // Note: The ID works w.r.t the sample vector
    let freq_mapping: Vec<(SampleID, _)> = freqs
        .iter()
        .enumerate()
        .map(|(i, freq)| (i as SampleID, freq))
        .collect();

    dbg!(&freq_mapping);
}
