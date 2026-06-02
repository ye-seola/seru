mod box_;
mod common;
mod image;
mod stack;
mod svg;
mod text;

pub use box_::{BoxDirection, BoxStyle, CrossAlign, MainAlign};
pub use common::CommonStyle;
pub use image::{ImageFit, ImageStyle};
pub use stack::{Place, StackStyle};
pub use svg::SvgStyle;
pub use text::{FontWeight, TextStyle};

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Percent(f32),
    Length(f32),
    Zero,
    Auto,
}

#[derive(Debug, Clone, Copy)]
pub struct EdgeInsets {
    pub top: Size,
    pub right: Size,
    pub bottom: Size,
    pub left: Size,
}

impl EdgeInsets {
    pub fn all(size: Size) -> EdgeInsets {
        EdgeInsets {
            top: size.clone(),
            right: size.clone(),
            bottom: size.clone(),
            left: size.clone(),
        }
    }

    pub fn vertical_horizontal(vertical: Size, horizontal: Size) -> EdgeInsets {
        EdgeInsets {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical.clone(),
            left: horizontal.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
