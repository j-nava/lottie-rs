use crate::{font::FontDB, Lottie};

pub struct WindowConfig {
    pub show_controls: bool,
    pub show_inspector: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Target {
    Default,
    Mask,
}

pub struct HeadlessConfig {
    pub target: Target,
    pub filename: String,
    pub frame: Option<u32>,
}

pub enum Config {
    Window(WindowConfig),
    Headless(HeadlessConfig),
}

/// The fundamental trait that every renderer need to implement
pub trait Renderer<F: FontDB> {
    /// Load a [Lottie] into this renderer
    fn load_lottie(&mut self, lottie: Lottie<F>, config: Config);
    /// Render the lottie file, possibly mutating self
    fn render(&mut self);
}
