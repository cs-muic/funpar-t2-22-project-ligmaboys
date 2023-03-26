use crate::{core::CoreState, data::vector2::Vector2};
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

    let ans = CoreState::process(&args.img_path, args.n_dimensions, args.width, args.height);


        
    // dbg!(&ans);

    let w = ans.iter().map(|arr| vec![arr[0], arr[1], arr[2]]).flatten().collect::<Vec<_>>();
    

    // Save the buffer as "image.png"
    image::save_buffer("image.png", &w, args.width as u32, args.height as u32, image::ColorType::Rgb8).unwrap()



    // dbg!(&cs.grid.get(Vector2{ x: 0, y: 1 }));
    // dbg!(&cs.grid.get(Vector2{ x: 1, y: 1 }));
    // dbg!(&cs.entropy_heap);

    // let least_entropy = &cs.entropy_heap.peek();
    // let least_entropy_pos = least_entropy.unwrap().coord;
    // dbg!(least_entropy);

    // dbg!(cs.choose_next_cell());
    // dbg!(least_entropy_pos);
}
