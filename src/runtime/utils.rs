use anyhow::Context;

use crate::{
    render::styles::{Color, EdgeInsets, Size},
    runtime::Value,
};

pub fn check_integer(num: f32) -> bool {
    if !num.is_finite() || num.fract() != 0.0 {
        return false;
    }

    return true;
}

pub fn check_non_negative_integer(num: f32) -> bool {
    if !check_integer(num) || num < 0.0 {
        return false;
    }

    return true;
}

impl Value {
    pub fn into_size(&self) -> anyhow::Result<Size> {
        match self {
            Value::Number(num) => Ok(Size::Length(*num)),

            Value::String(s) => Ok(string_to_size(s)?),

            _ => anyhow::bail!("invalid size value: {:?}", self),
        }
    }

    pub fn into_color(&self) -> anyhow::Result<Color> {
        let Value::String(s) = self else {
            anyhow::bail!("invalid color value: {:?}", self);
        };

        let s = s.to_lowercase();
        match s.as_str() {
            "red" => {
                return Ok(rgba(0xFF, 0, 0, 0xFF));
            }

            "green" => {
                return Ok(rgba(0, 0xFF, 0, 0xFF));
            }

            "blue" => {
                return Ok(rgba(0, 0, 0xFF, 0xFF));
            }

            v => Ok(parse_hex_color(v)?),
        }
    }

    pub fn into_insets(&self) -> anyhow::Result<EdgeInsets> {
        match self {
            Value::String(s) => {
                let sp = s.split_whitespace().collect::<Vec<_>>();
                match sp.len() {
                    1 => Ok(EdgeInsets::all(string_to_size(sp.get(0).unwrap())?)),
                    2 => Ok(EdgeInsets::vertical_horizontal(
                        string_to_size(sp.get(0).unwrap())?,
                        string_to_size(sp.get(1).unwrap())?,
                    )),
                    4 => Ok(EdgeInsets {
                        top: string_to_size(sp.get(0).unwrap())?,
                        right: string_to_size(sp.get(1).unwrap())?,
                        bottom: string_to_size(sp.get(2).unwrap())?,
                        left: string_to_size(sp.get(3).unwrap())?,
                    }),
                    _ => anyhow::bail!("invalid insets value: {}", s),
                }
            }
            Value::Number(num) => Ok(EdgeInsets::all(Size::Length(*num))),
            _ => anyhow::bail!("invalid insets value: {:?}", self),
        }
    }
}

fn string_to_size(s: &str) -> anyhow::Result<Size> {
    let s = s.trim();

    if s == "auto" {
        return Ok(Size::Auto);
    }

    if let Some(percent) = s.strip_suffix('%') {
        let num = percent.trim().parse::<f32>()?;
        return Ok(Size::Percent((num / 100.0).clamp(0.0, 1.0)));
    }

    let num = s
        .parse::<f32>()
        .with_context(|| format!("invalid size string: {}", s))?;

    Ok(Size::Length(num))
}

fn parse_hex_color(s: &str) -> anyhow::Result<Color> {
    let hex = s.trim().strip_prefix('#').unwrap_or(s.trim());

    match hex.len() {
        3 => Ok(rgba(
            hex_repeat(&hex[0..1])?,
            hex_repeat(&hex[1..2])?,
            hex_repeat(&hex[2..3])?,
            255,
        )),
        6 => Ok(rgba(
            hex_byte(&hex[0..2])?,
            hex_byte(&hex[2..4])?,
            hex_byte(&hex[4..6])?,
            255,
        )),
        8 => Ok(rgba(
            hex_byte(&hex[0..2])?,
            hex_byte(&hex[2..4])?,
            hex_byte(&hex[4..6])?,
            hex_byte(&hex[6..8])?,
        )),
        _ => anyhow::bail!("invalid hex color: {s:?}"),
    }
}

fn hex_byte(s: &str) -> anyhow::Result<u8> {
    Ok(u8::from_str_radix(s, 16)?)
}

fn hex_repeat(s: &str) -> anyhow::Result<u8> {
    hex_byte(&format!("{s}{s}"))
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}
