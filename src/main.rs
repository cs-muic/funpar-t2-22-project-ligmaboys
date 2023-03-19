use clap::Parser;
use cli::Args;
extern crate image;

mod cli;
mod data;
mod image_reader;
mod model;

fn main() {
    // Parse CLI <ImgPath> <Shape>
    let args: Args = Args::parse();

    let model = model::Model::create(&args.img_path, args.n_dimensions);

    dbg!(model.size());

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
