use clap::Parser;
use cli::Args;
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

    dbg!(samples);
}
