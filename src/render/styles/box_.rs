use std::str::FromStr;

use crate::render::{args::Args, styles::Size};

#[derive(Debug, Clone, Copy, Default)]
pub struct BoxStyle {
    pub gap: Option<Size>,
    pub direction: Option<BoxDirection>,
    pub main: Option<MainAlign>,
    pub cross: Option<CrossAlign>,
}

#[derive(Debug, Clone, Copy)]
pub enum MainAlign {
    Center,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Between,
    Around,
    Evenly,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub enum CrossAlign {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub enum BoxDirection {
    Column,
    Row,
}

impl BoxStyle {
    pub fn take_from(name: &str, args: &mut Args) -> anyhow::Result<Self> {
        let mut style = BoxStyle::default();

        if let Some(v) = args.take("gap") {
            style.gap = Some(v.into_size()?);
        }

        if let Some(v) = args.take_string("direction")? {
            style.direction = Some(v.parse()?);
        }

        if let Some(v) = args.take_string("main")? {
            style.main = Some(v.parse()?);
        }

        if let Some(v) = args.take_string("cross")? {
            style.cross = Some(v.parse()?);
        }

        apply_box_preset(name, &mut style);

        Ok(style)
    }
}

fn apply_box_preset(name: &str, style: &mut BoxStyle) {
    match name {
        "Center" => {
            style.direction.get_or_insert(BoxDirection::Column);
            style.main.get_or_insert(MainAlign::Center);
            style.cross.get_or_insert(CrossAlign::Center);
        }
        "Column" => {
            style.direction.get_or_insert(BoxDirection::Column);
        }
        "Row" => {
            style.direction.get_or_insert(BoxDirection::Row);
        }
        "Box" => {
            style.direction.get_or_insert(BoxDirection::Column);
            // style.main.get_or_insert(MainAlign::Start);
            style.cross.get_or_insert(CrossAlign::Stretch);
        }
        _ => {}
    }
}

impl FromStr for BoxDirection {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "column" => BoxDirection::Column,
            "row" => BoxDirection::Row,
            v => anyhow::bail!("invalid box direction: {}", v),
        })
    }
}

impl FromStr for MainAlign {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "center" => MainAlign::Center,
            "start" => MainAlign::Start,
            "end" => MainAlign::End,
            "flex-start" => MainAlign::FlexStart,
            "flex-end" => MainAlign::FlexEnd,
            "between" => MainAlign::Between,
            "around" => MainAlign::Around,
            "evenly" => MainAlign::Evenly,
            "stretch" => MainAlign::Stretch,
            v => anyhow::bail!("invalid main align: {}", v),
        })
    }
}

impl FromStr for CrossAlign {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "start" => CrossAlign::Start,
            "end" => CrossAlign::End,
            "flex-start" => CrossAlign::FlexStart,
            "flex-end" => CrossAlign::FlexEnd,
            "center" => CrossAlign::Center,
            "baseline" => CrossAlign::Baseline,
            "stretch" => CrossAlign::Stretch,
            v => anyhow::bail!("invalid cross align: {}", v),
        })
    }
}
