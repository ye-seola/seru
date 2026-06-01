use std::str::FromStr;

use crate::render::args::Args;

#[derive(Debug, Clone, Default)]
pub struct StackStyle {
    pub place_x: Option<Place>,
    pub place_y: Option<Place>,
}

#[derive(Debug, Clone, Copy)]
pub enum Place {
    Start,
    Center,
    End,
}

impl FromStr for Place {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "start" => Place::Start,
            "center" => Place::Center,
            "end" => Place::End,
            v => anyhow::bail!("invalid place: {}", v),
        })
    }
}

impl StackStyle {
    pub fn take_from(args: &mut Args) -> anyhow::Result<Self> {
        let mut stack_style = StackStyle::default();

        if let Some(v) = args.take_string("place_x")? {
            stack_style.place_x = Some(v.parse()?);
        }

        if let Some(v) = args.take_string("place_y")? {
            stack_style.place_y = Some(v.parse()?);
        }

        Ok(stack_style)
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
