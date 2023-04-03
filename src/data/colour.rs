use image::Rgba;

pub type Rgb = [u8; 3];

pub const BLACK: Rgb = [0, 0, 0];

pub fn make_rgb(rgb: &Rgba<u8>) -> Rgb {
    rgb.0[0..3].try_into().expect("RGB: Incorrect format")
}
