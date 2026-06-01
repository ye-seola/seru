use std::str::FromStr;

use crate::render::args::Args;

#[derive(Debug, Clone, Default)]
pub struct ImageStyle {
    pub fit: Option<ImageFit>,
}

impl ImageStyle {
    pub fn take_from(args: &mut Args) -> anyhow::Result<Self> {
        let mut image_style = ImageStyle::default();

        if let Some(v) = args.take_string("fit")? {
            image_style.fit = Some(v.parse()?);
        }

        Ok(image_style)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFit {
    Cover,
    Contain,
    Fill,
}

impl FromStr for ImageFit {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();

        Ok(match value.as_str() {
            "cover" => ImageFit::Cover,
            "contain" => ImageFit::Contain,
            "fill" => ImageFit::Fill,
            s => anyhow::bail!("invalid fit: {}", s),
        })
    }
}
