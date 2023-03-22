use clap::Parser;
use cli::Args;
extern crate image;

mod cli;
mod core;
mod data;
mod image_reader;
mod model;

fn main() {
    // Parse CLI <ImgPath> <Shape>
    let args: Args = Args::parse();

    let model = model::Model::create(&args.img_path, args.n_dimensions);

    dbg!(model.size());
}
