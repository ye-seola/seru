use crate::render::{args::Args, styles::Color};

#[derive(Debug, Clone, Default)]
pub struct SvgStyle {
    pub color: Option<Color>,
    pub stroke_width: Option<f32>,
}

impl SvgStyle {
    pub fn take_from(args: &mut Args) -> anyhow::Result<Self> {
        let mut svg_style = SvgStyle::default();
        if let Some(v) = args.take("color") {
            svg_style.color = Some(v.into_color()?);
        }
        if let Some(v) = args.take("stroke_width") {
            svg_style.stroke_width = Some(v.into_number()?);
        }

        Ok(svg_style)
    }
}
