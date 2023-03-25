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

    let mut cs = CoreState::new(&args.img_path, args.n_dimensions, args.width, args.height);

    dbg!(&cs.entropy_heap);

    let least_entropy = &cs.entropy_heap.peek();
    let least_entropy_pos = least_entropy.unwrap().coord;
    dbg!(least_entropy);

    dbg!(cs.choose_next_cell());
    dbg!(least_entropy_pos);
}
