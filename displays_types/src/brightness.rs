/// Brightness percentage in the inclusive range `0..=100`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Brightness(u8);

impl Brightness {
    pub const fn new(value: u8) -> Self {
        if value > 100 {
            panic!("Brightness needs to be between 0 and 100");
        }
        Self(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}
