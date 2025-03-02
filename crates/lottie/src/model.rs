pub use euclid::default::Rect;
pub use euclid::rect;
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
pub use serde_json::Error;
pub type Vector2D = euclid::default::Vector2D<f32>;

mod animated;
mod color;
mod helpers;

pub use animated::*;
pub use color::*;
use helpers::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    #[serde(rename = "nm")]
    pub name: Option<String>,
    #[serde(rename = "v", default)]
    version: Option<String>,
    #[serde(rename = "ip")]
    pub start_frame: f32,
    #[serde(rename = "op")]
    pub end_frame: f32,
    #[serde(rename = "fr")]
    pub frame_rate: f32,
    #[serde(rename = "w")]
    pub width: u32,
    #[serde(rename = "h")]
    pub height: u32,
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub fonts: FontList,
}

impl Model {
    pub fn from_reader<R: std::io::Read>(r: &mut R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(r)
    }

    pub fn duration(&self) -> f32 {
        (self.end_frame - self.start_frame) as f32 / self.frame_rate as f32
    }

    pub fn font(&self, name: &str) -> Option<&Font> {
        self.fonts.list.iter().find(|f| f.name == name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layer {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "ddd",
        default
    )]
    is_3d: bool,
    #[serde(rename = "hd", default)]
    pub hidden: bool,
    #[serde(rename = "ind", default)]
    pub index: Option<u32>,
    #[serde(rename = "parent", default)]
    pub parent_index: Option<u32>,
    #[serde(skip)]
    pub id: u32,
    #[serde(
        rename = "ao",
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub auto_orient: bool,
    #[serde(rename = "ip")]
    pub start_frame: f32,
    #[serde(rename = "op")]
    pub end_frame: f32,
    #[serde(rename = "st")]
    pub start_time: f32,
    #[serde(rename = "nm")]
    pub name: Option<String>,
    #[serde(rename = "ks", default)]
    pub transform: Option<Transform>,
    #[serde(flatten)]
    pub content: LayerContent,
    #[serde(rename = "tt", default)]
    pub matte_mode: Option<MatteMode>,
    #[serde(rename = "bm", default)]
    pub blend_mode: Option<BlendMode>,
    #[serde(default, rename = "hasMask")]
    pub has_mask: bool,
    #[serde(default, rename = "masksProperties")]
    pub masks_properties: Vec<Mask>,
}

impl Layer {
    pub fn time_remapping(&self) -> Option<Animated<f32>> {
        if let LayerContent::PreCompositionRef(pre) = &self.content {
            pre.time_remapping.clone()
        } else {
            None
        }
    }

    pub fn new(content: LayerContent, start_frame: f32, end_frame: f32, start_time: f32) -> Self {
        Layer {
            is_3d: false,
            hidden: false,
            index: None,
            parent_index: None,
            id: 0,
            auto_orient: false,
            start_frame,
            end_frame,
            start_time,
            name: None,
            transform: None,
            content,
            matte_mode: None,
            blend_mode: None,
            has_mask: false,
            masks_properties: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayerContent {
    PreCompositionRef(PreCompositionRef),
    SolidColor {
        color: Rgba,
        height: f32,
        width: f32,
    },
    MediaRef(MediaRef),
    Empty,
    Shape(ShapeGroup),
    #[cfg(feature = "text")]
    Text(TextAnimationData),
    Media(Media),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaRef {
    #[serde(rename = "refId")]
    pub ref_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreCompositionRef {
    #[serde(rename = "refId")]
    pub ref_id: String,
    #[serde(rename = "w")]
    width: u32,
    #[serde(rename = "h")]
    height: u32,
    #[serde(rename = "tm")]
    pub time_remapping: Option<Animated<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    #[serde(rename = "a", default)]
    pub anchor: Option<Animated<Vector2D>>,
    #[serde(rename = "p", default)]
    pub position: Option<Animated<Vector2D>>,
    #[serde(rename = "s", default = "default_vec2_100")]
    pub scale: Animated<Vector2D>,
    #[serde(rename = "r", default)]
    pub rotation: Animated<f32>,
    #[serde(skip)]
    pub auto_orient: bool,
    #[serde(rename = "o", default = "default_number_100")]
    pub opacity: Animated<f32>,
    #[serde(rename = "sk", default)]
    pub skew: Option<Animated<f32>>,
    #[serde(rename = "sa", default)]
    pub skew_axis: Option<Animated<f32>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            anchor: Default::default(),
            position: Default::default(),
            scale: default_vec2_100(),
            rotation: Default::default(),
            opacity: default_number_100(),
            skew: Default::default(),
            skew_axis: Default::default(),
            auto_orient: false,
        }
    }
}

impl Transform {
    pub fn is_identity(&self) -> bool {
        false
        // TODO:
    }

    pub fn frames(&self) -> f32 {
        let anchor_frames = self
            .anchor
            .as_ref()
            .and_then(|a| Some(a.keyframes.last()?.end_frame))
            .unwrap_or(0.0);
        let pos_frames = self
            .position
            .as_ref()
            .and_then(|a| Some(a.keyframes.last()?.end_frame))
            .unwrap_or(0.0);
        let scale_frames = self.scale.keyframes.last().unwrap().end_frame;
        let rotation_frames = self.rotation.keyframes.last().unwrap().end_frame;
        anchor_frames
            .max(pos_frames)
            .max(scale_frames)
            .max(rotation_frames)
    }

    pub fn initial_value(&self) -> Mat4 {
        self.value(0.0)
    }

    pub fn value(&self, frame: f32) -> Mat4 {
        let mut angle = 0.0;
        if let Some(position) = self.position.as_ref() {
            if self.auto_orient && position.is_animated() {
                let len = position.keyframes.len() - 1;
                let mut frame = position.keyframes[0].start_frame.max(frame);
                frame = position.keyframes[len].start_frame.min(frame);
                if let Some(keyframe) = position
                    .keyframes
                    .iter()
                    .find(|keyframe| frame >= keyframe.start_frame && frame < keyframe.end_frame)
                {
                    angle = (keyframe.end_value - keyframe.start_value)
                        .angle_from_x_axis()
                        .to_degrees();
                }
            }
        }
        let anchor = self
            .anchor
            .as_ref()
            .map(|a| a.value(frame))
            .unwrap_or_default();
        let position = self
            .position
            .as_ref()
            .map(|a| a.value(frame))
            .unwrap_or_default();
        let mut scale = self.scale.value(frame) / 100.0;
        let rotation = self.rotation.value(frame) + angle;
        // Some lottie file has scale = 0, which is invalid
        if scale.x == 0.0 {
            scale.x = f32::EPSILON;
        }
        if scale.y == 0.0 {
            scale.y = f32::EPSILON;
        }
        mat4(anchor, position, scale, rotation)
    }

    pub fn is_animated(&self) -> bool {
        self.anchor
            .as_ref()
            .map(|a| a.is_animated())
            .unwrap_or(false)
            || self
                .position
                .as_ref()
                .map(|a| a.is_animated())
                .unwrap_or(false)
            || self.scale.is_animated()
            || self.rotation.is_animated()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepeaterTransform {
    #[serde(rename = "a", default)]
    anchor: Animated<Vector2D>,
    #[serde(rename = "p")]
    position: Animated<Vector2D>,
    #[serde(rename = "s")]
    scale: Animated<Vector2D>,
    #[serde(rename = "r")]
    rotation: Animated<f32>,
    #[serde(rename = "so")]
    start_opacity: Animated<f32>,
    #[serde(rename = "eo")]
    end_opacity: Animated<f32>,
    #[serde(rename = "sk", default)]
    skew: Option<Animated<Vector2D>>,
    #[serde(rename = "sa", default)]
    skew_axis: Option<Animated<Vector2D>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FontList {
    pub list: Vec<Font>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Font {
    #[serde(default)]
    ascent: Option<f32>,
    #[serde(rename = "fFamily")]
    pub family: String,
    #[serde(rename = "fName")]
    pub name: String,
    #[serde(rename = "fStyle")]
    style: String,
    #[serde(rename = "fPath", default)]
    pub path: Option<String>,
    #[serde(rename = "fWeight")]
    weight: Option<String>,
    #[serde(default)]
    pub origin: FontPathOrigin,
    #[serde(rename = "fClass", default)]
    class: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShapeLayer {
    #[serde(rename = "nm", default)]
    pub name: Option<String>,
    #[serde(rename = "hd", default)]
    pub hidden: bool,
    #[serde(flatten)]
    pub shape: Shape,
}

#[derive(Debug, Clone)]
pub struct TextRangeInfo {
    pub value: Vec<Vec<char>>,
    pub index: (usize, usize), // line, char
    pub ranges: Vec<TextRange>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "ty")]
pub enum Shape {
    #[serde(rename = "rc")]
    Rectangle(Rectangle),
    #[serde(rename = "el")]
    Ellipse(Ellipse),
    #[serde(rename = "sr")]
    PolyStar(PolyStar),
    #[serde(rename = "sh")]
    Path {
        #[serde(rename = "ks")]
        d: Animated<Vec<Bezier>>,
        #[serde(skip)]
        text_range: Option<TextRangeInfo>,
    },
    #[serde(rename = "fl")]
    Fill(Fill),
    #[serde(rename = "st")]
    Stroke(Stroke),
    #[serde(rename = "gf")]
    GradientFill(GradientFill),
    #[serde(rename = "gs")]
    GradientStroke(GradientStroke),
    #[serde(rename = "gr")]
    Group {
        // TODO: add np property
        #[serde(rename = "it")]
        shapes: Vec<ShapeLayer>,
    },
    #[serde(rename = "tr")]
    Transform(Transform),
    #[serde(rename = "rp")]
    Repeater {
        #[serde(rename = "c")]
        copies: Animated<f32>,
        #[serde(rename = "o")]
        offset: Animated<f32>,
        #[serde(rename = "m")]
        composite: Composite,
        #[serde(rename = "tr")]
        transform: RepeaterTransform,
    },
    #[serde(rename = "tm")]
    Trim(Trim),
    #[serde(rename = "rd")]
    RoundedCorners {
        #[serde(rename = "r")]
        radius: Animated<f32>,
    },
    #[serde(rename = "pb")]
    PuckerBloat {
        #[serde(rename = "a")]
        amount: Animated<f32>,
    },
    #[serde(rename = "tw")]
    Twist {
        #[serde(rename = "a")]
        angle: Animated<f32>,
        #[serde(rename = "c")]
        center: Animated<Vector2D>,
    },
    #[serde(rename = "mm")]
    Merge {
        #[serde(rename = "mm")]
        mode: MergeMode,
    },
    #[serde(rename = "op")]
    OffsetPath {
        #[serde(rename = "a")]
        amount: Animated<f32>,
        #[serde(rename = "lj")]
        line_join: LineJoin,
        #[serde(rename = "ml")]
        miter_limit: f32,
    },
    #[serde(rename = "zz")]
    ZigZag {
        #[serde(rename = "r")]
        radius: Animated<f32>,
        #[serde(rename = "s")]
        distance: Animated<f32>,
        #[serde(rename = "pt")]
        ridges: Animated<f32>,
    },
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum PolyStarType {
    Star = 1,
    Polygon = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum FillRule {
    NonZero = 1,
    EvenOdd = 2,
}

impl Default for FillRule {
    fn default() -> Self {
        FillRule::NonZero
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineCap {
    Butt = 1,
    Round = 2,
    Square = 3,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineJoin {
    Miter = 1,
    Round = 2,
    Bevel = 3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrokeDash {
    #[serde(rename = "v")]
    length: Animated<f32>,
    #[serde(rename = "n")]
    ty: StrokeDashType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StrokeDashType {
    #[serde(rename = "d")]
    Dash,
    #[serde(rename = "g")]
    Gap,
    #[serde(rename = "o")]
    Offset,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum GradientType {
    Linear = 1,
    Radial = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Composite {
    Above = 1,
    Below = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TrimMultipleShape {
    Individually = 1,
    Simultaneously = 2,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum ShapeDirection {
    Clockwise = 1,
    CounterClockwise = 2,
}

impl Default for ShapeDirection {
    fn default() -> Self {
        ShapeDirection::Clockwise
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum MergeMode {
    #[serde(other)]
    Unsupported = 1,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum FontPathOrigin {
    Local = 0,
    CssUrl = 1,
    ScriptUrl = 2,
    FontUrl = 3,
}

impl Default for FontPathOrigin {
    fn default() -> Self {
        FontPathOrigin::Local
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextJustify {
    Left = 0,
    Right = 1,
    Center = 2,
    LastLineLeft = 3,
    LastLineRight = 4,
    LastLineCenter = 5,
    LastLineFull = 6,
}

impl Default for TextJustify {
    fn default() -> Self {
        TextJustify::Left
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextCaps {
    Regular = 0,
    AllCaps = 1,
    SmallCaps = 2,
}

impl Default for TextCaps {
    fn default() -> Self {
        TextCaps::Regular
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextBased {
    Characters = 1,
    CharactersExcludingSpaces = 2,
    Words = 3,
    Lines = 4,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextShape {
    Square = 1,
    RampUp = 2,
    RampDown = 3,
    Triangle = 4,
    Round = 5,
    Smooth = 6,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum MatteMode {
    Normal = 0,
    Alpha = 1,
    InvertedAlpha = 2,
    Luma = 3,
    InvertedLuma = 4,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum BlendMode {
    Normal = 0,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HighLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Add,
    HardMix,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trim {
    #[serde(rename = "s")]
    pub start: Animated<f32>,
    #[serde(rename = "e")]
    pub end: Animated<f32>,
    #[serde(rename = "o")]
    pub offset: Animated<f32>,
    #[serde(rename = "m")]
    pub multiple_shape: TrimMultipleShape,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fill {
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "c")]
    pub color: Animated<Rgb>,
    #[serde(rename = "r", default)]
    pub fill_rule: FillRule,
}

impl Fill {
    pub fn transparent() -> Fill {
        Fill {
            opacity: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(0.0)],
            },
            color: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(Rgb::new_u8(0, 0, 0))],
            },
            fill_rule: FillRule::NonZero,
        }
    }
}

impl From<Rgba> for Fill {
    fn from(color: Rgba) -> Self {
        Fill {
            opacity: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(color.a as f32 / 255.0)],
            },
            color: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(Rgb::new_u8(color.r, color.g, color.b))],
            },
            fill_rule: FillRule::NonZero,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stroke {
    #[serde(rename = "lc")]
    pub line_cap: LineCap,
    #[serde(rename = "lj")]
    pub line_join: LineJoin,
    #[serde(rename = "ml", default)]
    miter_limit: f32,
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "w")]
    pub width: Animated<f32>,
    #[serde(rename = "d", default)]
    dashes: Vec<StrokeDash>,
    #[serde(rename = "c")]
    pub color: Animated<Rgb>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "ColorListHelper", into = "ColorListHelper")]
pub struct ColorList {
    color_count: usize,
    pub colors: Animated<Vec<GradientColor>>,
}

#[derive(Debug, Clone)]
pub struct GradientColor {
    pub offset: f32,
    pub color: Rgba,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gradient {
    #[serde(rename = "s")]
    pub start: Animated<Vector2D>,
    #[serde(rename = "e")]
    pub end: Animated<Vector2D>,
    #[serde(rename = "t")]
    pub gradient_ty: GradientType,
    #[serde(rename = "g")]
    pub colors: ColorList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GradientFill {
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "r")]
    pub fill_rule: FillRule,
    #[serde(flatten)]
    pub gradient: Gradient,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GradientStroke {
    #[serde(rename = "lc")]
    pub line_cap: LineCap,
    #[serde(rename = "lj")]
    pub line_join: LineJoin,
    #[serde(rename = "ml")]
    miter_limit: f32,
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "w")]
    pub width: Animated<f32>,
    #[serde(rename = "d", default)]
    dashes: Vec<StrokeDash>,
    #[serde(flatten)]
    pub gradient: Gradient,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rectangle {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "s")]
    pub size: Animated<Vector2D>,
    #[serde(rename = "r")]
    pub radius: Animated<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ellipse {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "s")]
    pub size: Animated<Vector2D>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolyStar {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "or")]
    pub outer_radius: Animated<f32>,
    #[serde(rename = "os")]
    pub outer_roundness: Animated<f32>,
    #[serde(rename = "ir", default)]
    pub inner_radius: Option<Animated<f32>>,
    #[serde(rename = "is")]
    pub inner_roundness: Option<Animated<f32>>,
    #[serde(rename = "r")]
    pub rotation: Animated<f32>,
    #[serde(rename = "pt")]
    pub points: Animated<f32>,
    #[serde(rename = "sy")]
    pub star_type: PolyStarType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Asset {
    Media(Media),
    Sound,
    Precomposition(Precomposition),
}

impl Asset {
    pub fn id(&self) -> &str {
        match self {
            Asset::Media(i) => i.id.as_str(),
            Asset::Precomposition(p) => p.id.as_str(),
            _ => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
    #[serde(rename = "u", default)]
    pub pwd: String,
    #[serde(rename = "p")]
    pub filename: String,
    #[serde(
        rename = "e",
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub embedded: bool,
    id: String,
    #[serde(rename = "nm", default)]
    name: Option<String>,
    #[serde(rename = "w", default)]
    pub width: Option<u32>,
    #[serde(rename = "h", default)]
    pub height: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Precomposition {
    pub id: String,
    pub layers: Vec<Layer>,
    #[serde(rename = "nm")]
    name: Option<String>,
    #[serde(rename = "fr")]
    pub frame_rate: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShapeGroup {
    pub shapes: Vec<ShapeLayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Bezier {
    #[serde(rename = "c", default)]
    pub closed: bool,
    #[serde(
        rename = "v",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub verticies: Vec<Vector2D>,
    #[serde(
        rename = "i",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub in_tangent: Vec<Vector2D>,
    #[serde(
        rename = "o",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub out_tangent: Vec<Vector2D>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextAnimationData {
    #[serde(rename = "a")]
    pub ranges: Vec<TextRange>,
    #[serde(rename = "d")]
    pub document: TextData,
    #[serde(rename = "m")]
    options: TextAlignmentOptions,
    #[serde(rename = "p")]
    follow_path: TextFollowPath,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextStyle {
    #[serde(rename = "sw", default)]
    stroke_width: Option<Animated<f32>>,
    #[serde(rename = "sc", default)]
    stroke_color: Option<Animated<Rgb>>,
    #[serde(rename = "sh", default)]
    stroke_hue: Option<Animated<f32>>,
    #[serde(rename = "ss", default)]
    stroke_saturation: Option<Animated<f32>>,
    #[serde(rename = "sb", default)]
    stroke_brightness: Option<Animated<f32>>,
    #[serde(rename = "so", default)]
    stroke_opacity: Option<Animated<f32>>,
    #[serde(rename = "fc", default)]
    fill_color: Option<Animated<Rgb>>,
    #[serde(rename = "fh", default)]
    fill_hue: Option<Animated<f32>>,
    #[serde(rename = "fs", default)]
    fill_saturation: Option<Animated<f32>>,
    #[serde(rename = "fb", default)]
    fill_brightness: Option<Animated<f32>>,
    #[serde(rename = "t", default)]
    pub letter_spacing: Option<Animated<f32>>,
    #[serde(rename = "bl", default)]
    blur: Option<Animated<f32>>,
    #[serde(rename = "ls", default)]
    pub line_spacing: Option<Animated<f32>>,
    #[serde(flatten)]
    transform: Option<Transform>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextRange {
    #[serde(rename = "nm", default)]
    name: Option<String>,
    #[serde(rename = "a", default)]
    pub style: Option<TextStyle>,
    #[serde(rename = "s")]
    pub selector: TextRangeSelector,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextRangeSelector {
    #[serde(rename = "t", deserialize_with = "bool_from_int")]
    expressible: bool,
    #[serde(rename = "xe")]
    max_ease: Animated<f32>,
    #[serde(rename = "ne")]
    min_ease: Animated<f32>,
    #[serde(rename = "a")]
    max_amount: Animated<f32>,
    #[serde(rename = "b")]
    based_on: TextBased,
    #[serde(rename = "rn", deserialize_with = "bool_from_int")]
    randomize: bool,
    #[serde(rename = "sh")]
    shape: TextShape,
    #[serde(rename = "o", default)]
    offset: Option<Animated<f32>>,
    #[serde(rename = "r")]
    pub range_units: TextBased,
    #[serde(rename = "sm", default)]
    selector_smoothness: Option<Animated<f32>>,
    #[serde(rename = "s", default)]
    pub start: Option<Animated<f32>>,
    #[serde(rename = "e", default)]
    pub end: Option<Animated<f32>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TextData {
    #[serde(rename = "x", default)]
    expression: Option<String>,
    #[serde(
        deserialize_with = "keyframes_from_array",
        serialize_with = "array_from_keyframes",
        rename = "k"
    )]
    pub keyframes: Vec<KeyFrame<TextDocument>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextAlignmentOptions {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextFollowPath {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextDocument {
    #[serde(rename = "t")]
    pub value: String,
    #[serde(rename = "f")]
    pub font_name: String,
    #[serde(rename = "s")]
    pub size: f32,
    #[serde(
        rename = "fc",
        deserialize_with = "array_to_rgba",
        serialize_with = "array_from_rgba",
        default
    )]
    pub fill_color: Rgba,
    #[serde(
        rename = "sc",
        deserialize_with = "array_to_rgba",
        serialize_with = "array_from_rgba",
        default
    )]
    stroke_color: Rgba,
    #[serde(rename = "sw", default)]
    stroke_width: f32,
    #[serde(rename = "of", default)]
    stroke_above_fill: bool,
    #[serde(rename = "lh", default)]
    line_height: Option<f32>,
    #[serde(rename = "j", default)]
    pub justify: TextJustify,
    #[serde(rename = "ls", default)]
    pub baseline_shift: f32,
    // TODO:
    #[serde(default)]
    sz: Vec<f32>,
    #[serde(default)]
    ps: Vec<f32>,
    #[serde(default)]
    ca: TextCaps,
}

impl Default for TextDocument {
    fn default() -> Self {
        TextDocument {
            font_name: String::new(),
            size: 14.0,
            fill_color: Rgba::new_u8(0, 0, 0, 255),
            stroke_color: Rgba::new_u8(0, 0, 0, 255),
            stroke_width: 0.0,
            stroke_above_fill: false,
            line_height: None,
            baseline_shift: 0.0,
            value: String::new(),
            justify: TextJustify::Left,
            sz: vec![],
            ps: vec![],
            ca: TextCaps::Regular,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mask {
    #[serde(rename = "nm", default)]
    pub name: String,
    #[serde(rename = "mn", default)]
    match_name: String,
    #[serde(rename = "inv", default)]
    inverted: bool,
    #[serde(rename = "pt")]
    pub points: Animated<Vec<Bezier>>,
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    pub mode: MaskMode,
    #[serde(rename = "e", default)]
    expand: Option<Animated<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MaskMode {
    #[serde(rename = "n")]
    None,
    #[serde(rename = "a")]
    Add,
    #[serde(rename = "s")]
    Subtract,
    #[serde(rename = "i")]
    Intersect,
    #[serde(rename = "l")]
    Lighten,
    #[serde(rename = "d")]
    Darken,
    #[serde(rename = "f")]
    Difference,
}

fn mat4(anchor: Vector2D, position: Vector2D, scale: Vector2D, rotation: f32) -> Mat4 {
    let anchor = Vec3::new(anchor.x, anchor.y, 0.0);
    let scale = Vec3::new(scale.x, scale.y, 1.0);
    let position = Vec3::new(position.x, position.y, 0.0);
    Mat4::from_translation(position)
        * Mat4::from_rotation_z(rotation * std::f32::consts::PI / 180.0)
        * Mat4::from_scale(scale)
        * Mat4::from_translation(-anchor)
}
