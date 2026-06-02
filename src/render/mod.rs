mod args;
pub mod components;
pub mod context;
pub mod registry;
mod skia;
pub mod styles;

use crate::render::styles::*;

pub use skia::render;

#[derive(Debug, Clone)]
pub struct RenderNode {
    pub kind: RenderNodeKind,
    pub children: Vec<RenderNode>,
    pub style: CommonStyle,
}

#[derive(Debug, Clone)]
pub enum RenderNodeKind {
    Box {
        style: BoxStyle,
    },
    Stack {
        style: StackStyle,
    },
    Text {
        style: TextStyle,
        text: String,
    },
    Image {
        style: ImageStyle,
        image: Option<skia_safe::Image>,
    },
    Svg {
        style: SvgStyle,
        svg: Option<skia_safe::svg::Dom>,
    },
}

pub enum RenderOutputType {
    SVG,
    PNG,
}

pub struct RenderOptions {
    pub output_type: RenderOutputType,
    pub background: Option<Color>,
    pub render_scale: Option<f32>,
    pub width: f32,
    pub height: f32,
}
