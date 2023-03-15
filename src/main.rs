use clap::Parser;
use cli::Args;
extern crate image;

mod cli;

fn main() {
    // Parse CLI
    let args = Args::parse();

    // Load image from args passed in
    let img = image::open(args.img_path).expect("Failed to open image");

    // Process the image
    dbg!(img);
}
