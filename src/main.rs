use crate::core::CoreState;
use clap::Parser;
use cli::Args;
extern crate image;

mod cli;
mod core;
mod data;
mod entropy_coord;
mod image_reader;
mod model;

fn main() {
    // Parse CLI <ImgPath> <Shape> <OutputWidth> <OutputHeight>
    let args: Args = Args::parse();

    let ans = CoreState::process(
        &args.img_path,
        args.n_dimensions,
        args.width,
        args.height,
        args.rotation,
    );
    let w = ans
        .iter()
        .flat_map(|arr| vec![arr[0], arr[1], arr[2]])
        .collect::<Vec<_>>();

    // Save the buffer as "image.png"
    image::save_buffer(
        "image.png",
        &w,
        args.width as u32,
        args.height as u32,
        image::ColorType::Rgb8,
    )
    .unwrap()
}
