use crate::render::{
    args::Args,
    styles::{Color, EdgeInsets, Size},
};

#[derive(Debug, Clone, Default)]
pub struct CommonStyle {
    pub radius: Option<f32>,

    pub width: Option<Size>,
    pub height: Option<Size>,

    pub max_width: Option<Size>,
    pub max_height: Option<Size>,

    pub min_width: Option<Size>,
    pub min_height: Option<Size>,

    pub margin: Option<EdgeInsets>,
    pub padding: Option<EdgeInsets>,
    pub border: Option<f32>,
    pub border_color: Option<Color>,
    pub background: Option<Color>,

    // flex
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<f32>,
}

impl CommonStyle {
    pub fn take_from(args: &mut Args) -> anyhow::Result<Self> {
        let mut style = CommonStyle::default();

        if let Some(v) = args.take("width") {
            style.width = Some(v.into_size()?);
        }
        if let Some(v) = args.take("height") {
            style.height = Some(v.into_size()?);
        }
        if let Some(v) = args.take("max_width") {
            style.max_width = Some(v.into_size()?);
        }
        if let Some(v) = args.take("max_height") {
            style.max_height = Some(v.into_size()?);
        }
        if let Some(v) = args.take("min_width") {
            style.min_width = Some(v.into_size()?);
        }
        if let Some(v) = args.take("min_height") {
            style.min_height = Some(v.into_size()?);
        }
        if let Some(v) = args.take("margin") {
            style.margin = Some(v.into_insets()?);
        }
        if let Some(v) = args.take("padding") {
            style.padding = Some(v.into_insets()?);
        }
        if let Some(v) = args.take("border") {
            style.border = Some(v.into_number()?);
        }
        if let Some(v) = args.take("border_color") {
            style.border_color = Some(v.into_color()?);
        }
        if let Some(v) = args.take("background") {
            style.background = Some(v.into_color()?);
        }
        if let Some(v) = args.take("radius") {
            style.radius = Some(v.into_number()?);
        }
        if let Some(v) = args.take("grow") {
            style.grow = Some(v.into_number()?);
        }
        if let Some(v) = args.take("shrink") {
            style.shrink = Some(v.into_number()?);
        }
        if let Some(v) = args.take("basis") {
            style.basis = Some(v.into_number()?);
        }

        Ok(style)
    }
}
