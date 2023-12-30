#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
    width: u16,
    height: u16,
}

impl Dimension {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}
