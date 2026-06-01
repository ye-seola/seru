use std::str::FromStr;

use crate::{
    render::{args::Args, styles::Color},
    runtime::utils::check_non_negative_integer,
};

#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_size: Option<f32>,
    pub color: Option<Color>,
    pub ellipsis: Option<bool>,
    pub max_lines: Option<usize>,
}

impl TextStyle {
    pub fn take_from(args: &mut Args) -> anyhow::Result<Self> {
        let mut text_style = TextStyle::default();

        if let Some(v) = args.take_string("font_family")? {
            text_style.font_family = Some(v);
        }

        if let Some(v) = args.take("font_weight") {
            text_style.font_weight = Some(v.into_string()?.parse()?);
        }

        if let Some(v) = args.take_number("font_size")? {
            text_style.font_size = Some(v);
        }

        if let Some(v) = args.take("color") {
            text_style.color = Some(v.into_color()?);
        }

        if let Some(v) = args.take("ellipsis") {
            text_style.ellipsis = Some(v.into_bool()?);
        }

        if let Some(v) = args.take_number("max_lines")? {
            if !check_non_negative_integer(v) {
                anyhow::bail!("must be non-negative integer: {}", v);
            }

            text_style.max_lines = Some(v as usize);
        }

        Ok(text_style)
    }
}

#[derive(Debug, Clone)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
}

impl FromStr for FontWeight {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();

        Ok(match value.as_str() {
            "thin" => FontWeight::Thin,
            "extra_light" | "extralight" => FontWeight::ExtraLight,
            "light" => FontWeight::Light,
            "normal" => FontWeight::Normal,
            "medium" => FontWeight::Medium,
            "semibold" => FontWeight::SemiBold,
            "bold" => FontWeight::Bold,
            "extra_bold" | "extrabold" => FontWeight::ExtraBold,
            "black" => FontWeight::Black,
            "extra_black" | "extrablack" => FontWeight::ExtraBlack,
            s => anyhow::bail!("invalid font weight: {}", s),
        })
    }
}
