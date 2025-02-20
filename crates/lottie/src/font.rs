use std::io::Read;

use crate::model::{Font as LottieFont, Model};
#[cfg(feature = "text")]
use fontkit::{Font, FontKey, FontKit};

use crate::Error;

const BUFFER_LIMIT: usize = 10 * 1_024 * 1_024;

pub trait FontDB {
    type FontKit;
    type FontKey;
    type Font;
    fn new(fontkit: Self::FontKit) -> Self;
    fn load_fonts_from_model(&mut self, model: &Model) -> Result<(), Error>;
    fn font(&self, font: &LottieFont) -> Option<&Self::Font>;
    fn fontkit(&self) -> &Self::FontKit;
    fn from_reader<R: Read>(r: &mut R, root_path: &str) -> Result<Self, Error> where Self:Sized;
}

pub struct MockFont;

impl FontDB for MockFont {
    type FontKit = ();
    type FontKey = ();
    type Font = ();
    fn new(_: Self::FontKit) -> Self {
        Self
    }

    fn load_fonts_from_model(&mut self, _: &Model) -> Result<(), Error> {
        Ok(())
    }

    fn font(&self, _: &LottieFont) -> Option<&Self::Font> {
        None
    }

    fn fontkit(&self) -> &Self::FontKit {
        &()
    }

    fn from_reader<R: Read>(_: &mut R, _: &str) -> Result<Self, Error> {
        Ok(Self)
    }
}

#[cfg(feature = "text")]
pub struct FontKitDB {
    fontkit: FontKit,
    font_map: HashMap<String, Vec<FontKey>>,
}

#[cfg(feature = "text")]
impl FontDB for FontKitDB {
    type FontKit = FontKit;
    type FontKey = FontKey;
    type Font = Font;
    fn new(fontkit: FontKit) -> Self {
        Self {
            fontkit,
            font_map: std::collections::HashMap::new(),
        }
    }

    #[cfg(not(all(target_os = "unknown", target_arch = "wasm32")))]
    pub fn from_reader<R: Read>(r: R, root_path: &str) -> Result<Self, Error> {
        let fontkit = FontKit::new();
        let path = dirs::font_dir().unwrap();
        fontkit.search_fonts_from_path(path)?;
        #[cfg(target_os = "macos")]
        fontkit.search_fonts_from_path(std::path::PathBuf::from("/System/Library/Fonts"))?;
        let model = Model::from_reader(r)?;
        Ok(Lottie::new(model, fontkit, root_path)?)
    }


    fn load_fonts_from_model(&mut self, model: &Model) -> Result<(), Error> {
        // load default font
        #[cfg(not(target_arch = "wasm32"))]
        {
            let current_exe = std::env::current_exe()?;
            let mut path = current_exe.clone();
            path.push("assets/FiraMono-Regular.ttf");

            while !path.exists() && path.parent().is_some() {
                path.pop();
                path.pop();
                path.pop();
                path.push("assets/FiraMono-Regular.ttf");
            }
            if path.exists() {
                self.fontkit.search_fonts_from_path(
                    &path.into_os_string().into_string().unwrap_or_default(),
                )?;
            }
        }
        // load remote fonts
        for font in &model.fonts.list {
            if let Some(path) = font.path.as_ref() {
                if font.origin == crate::model::FontPathOrigin::FontUrl {
                    let response = ureq::get(path).call()?;
                    let mut bytes = vec![];
                    response
                        .into_reader()
                        .take((BUFFER_LIMIT + 1) as u64)
                        .read_to_end(&mut bytes)?;
                    let keys = self.fontkit.add_font_from_buffer(bytes)?;
                    self.font_map.insert(font.name.clone(), keys);
                }
            }
        }
        Ok(())
    }

    fn font(&self, font: &LottieFont) -> Option<impl Deref<Target = Font> + '_> {
        match font.origin {
            // This is not an html player. So we treat script/css urls as local obtained fonts
            // TODO: could this be a thing in WASM target?
            FontPathOrigin::Local | FontPathOrigin::ScriptUrl | FontPathOrigin::CssUrl => self
                .fontkit
                .query(&FontKey::new_with_family(font.name.clone()))
                .or_else(|| {
                    self.fontkit
                        .query(&FontKey::new_with_family(font.family.clone()))
                })
                .or_else(|| {
                    // default font
                    self.fontkit
                        .query(&FontKey::new_with_family("Fira Mono".to_string()))
                }),
            // TODO: What if font from url is *.ttc and font.name points to one font in the
            // collection? Could this be possible?
            FontPathOrigin::FontUrl => self.fontkit.query(self.font_map.get(&font.name)?.first()?),
        }
    }

    fn fontkit(&self) -> &FontKit {
        &self.fontkit
    }
}
