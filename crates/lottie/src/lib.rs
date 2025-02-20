use std::io::Read;
use crate::model::Model;
pub use error::Error;
pub use font::{FontDB, MockFont};
#[cfg(feature = "text")]
pub use fontkit::tiny_skia_path;
#[cfg(feature = "text")]
use fontkit::FontKit;
pub use lerp::*;
pub use renderer::*;
use timeline::Timeline;

mod error;
mod font;
mod layer;
mod lerp;
mod model;
mod renderer;
mod timeline;


pub mod prelude {
    pub use crate::layer::frame::*;
    pub use crate::layer::hierarchy::*;
    pub use crate::layer::shape::{
        AnyFill, AnyStroke, PathFactory, StyledShape, StyledShapeIterator, TrimInfo,
    };
    pub use crate::layer::staged::{RenderableContent, StagedLayer};
    pub use crate::model::*;
    pub use crate::timeline::{Id, TimelineAction};
}

pub struct Lottie<F: FontDB> {
    pub model: Model,
    pub scale: f32,
    fontdb: F,
    timeline: Timeline,
}

impl<F: FontDB> Lottie<F> {
    /// Initiate a new `Lottie` by providing a raw `Model`, a `FontKit` for font
    /// management, and a root path.Root path will be used to resolve relative
    /// paths of media files in this lottie model
    pub fn new(model: Model, mut fontdb: F, root_path: &str) -> Result<Self, Error> {
        fontdb.load_fonts_from_model(&model)?;

        let timeline = Timeline::new(&model, &fontdb, root_path)?;
        Ok(Lottie {
            model,
            timeline,
            fontdb,
            scale: 1.0,
        })
    }

    #[cfg(not(all(target_os = "unknown", target_arch = "wasm32")))]
    pub fn from_reader<R: Read>(r: &mut R, root_path: &str) -> Result<Self, Error> {
        let font = F::from_reader(r, root_path)?;
        let model = Model::from_reader(r)?;
        Ok(Lottie::new(model, font, root_path)?)
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn fontdb(&self) -> &F {
        &self.fontdb
    }
}
