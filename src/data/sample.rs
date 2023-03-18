use super::colour::Rgb;

//
// Sample Container
//
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sample {
    pub region: Vec<Rgb>,
}

pub type SampleID = usize;

impl Sample {
    pub fn new() -> Sample {
        Sample { region: Vec::new() }
    }
}
