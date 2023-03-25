use crate::core::CoreState;
use clap::Parser;
use cli::Args;
extern crate image;

mod cli;
mod core;
mod data;
mod image_reader;
mod model;

fn main() {
    // Parse CLI <ImgPath> <Shape> <OutputWidth> <OutputHeight>
    let args: Args = Args::parse();

    let cs = CoreState::new(&args.img_path, args.n_dimensions, args.width, args.height);

    dbg!(&cs.grid);
    println!(
        "{:?}",
        cs.grid
            .data
            .iter()
            .map(|cell| cell.entropy())
            .collect::<Vec<_>>()
    )
}
