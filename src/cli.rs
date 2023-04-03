use clap::Parser;

#[derive(Parser, Default, Debug)]
pub struct Args {
    // Name of the image file
    pub img_path: String,
    pub n_dimensions: usize,
    pub width: usize,
    pub height: usize,
    #[arg(long)]
    pub rotation: bool,
}
