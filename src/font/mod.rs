use anyhow::Context;
use skia_safe::{
    Color as SKColor, FontMgr, FontStyle,
    font_style::{Slant, Weight, Width},
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as SkiaTextStyle,
        TypefaceFontProvider,
    },
};
use std::borrow::Cow;

use crate::render::{styles::FontWeight, styles::TextStyle};

#[derive(Debug)]
pub enum Font {
    Path(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct FontFile {
    pub font: Font,
    pub alias: Option<String>,
}

pub struct FontManager {
    font_collection: FontCollection,
    font_mgr: FontMgr,
}

impl FontManager {
    pub fn new(load_system_fonts: bool, fonts: &[FontFile]) -> anyhow::Result<FontManager> {
        let mut font_collection = FontCollection::new();

        let mut ffont_mgr: Option<FontMgr> = None;

        if load_system_fonts {
            let mgr = FontMgr::new();
            font_collection.set_default_font_manager(mgr.clone(), None);
            ffont_mgr = Some(mgr);
        }

        if fonts.len() > 0 {
            let font_mgr = FontMgr::custom_empty().with_context(|| "FontMgr::custom_apply fail")?;
            let mut provider = TypefaceFontProvider::new();

            for font in fonts {
                let (font_data, font_name) = match &font.font {
                    Font::Path(path) => (Cow::Owned(std::fs::read(path)?), path.to_string()),
                    Font::Bytes(data) => (Cow::Borrowed(data), "[bytes]".to_string()),
                };

                let typeface = font_mgr
                    .new_from_data(font_data.as_ref(), None)
                    .ok_or_else(|| anyhow::anyhow!("failed to load font: {}", font_name))?;
                provider.register_typeface(typeface, font.alias.clone().as_deref());
            }

            let mgr = FontMgr::from(provider);
            font_collection.set_asset_font_manager(mgr.clone());
            ffont_mgr = Some(mgr);
        }

        Ok(FontManager {
            font_collection,
            font_mgr: ffont_mgr.unwrap_or_else(|| FontMgr::empty()),
        })
    }

    pub fn get_font_mgr(&self) -> FontMgr {
        self.font_mgr.clone()
    }

    pub fn create_paragraph(&self, style: &TextStyle, text: &str) -> Paragraph {
        let mut paragraph_style = ParagraphStyle::new();

        {
            if let Some(true) = style.ellipsis {
                paragraph_style.set_ellipsis("\u{2026}");
            }

            if let Some(lines) = style.max_lines {
                paragraph_style.set_max_lines(lines);
            }
        }

        {
            let mut text_style = SkiaTextStyle::new();
            text_style.set_subpixel(true);

            if let Some(family) = &style.font_family {
                text_style.set_font_families(&[family]);
            }

            if let Some(size) = style.font_size {
                text_style.set_font_size(size);
            }

            if let Some(color) = &style.color {
                text_style.set_color(*color);
            } else {
                text_style.set_color(SKColor::BLACK);
            }

            if let Some(weight) = &style.font_weight {
                let weight = match weight {
                    FontWeight::Thin => Weight::THIN,
                    FontWeight::ExtraLight => Weight::EXTRA_LIGHT,
                    FontWeight::Light => Weight::LIGHT,
                    FontWeight::Normal => Weight::NORMAL,
                    FontWeight::Medium => Weight::MEDIUM,
                    FontWeight::SemiBold => Weight::SEMI_BOLD,
                    FontWeight::Bold => Weight::BOLD,
                    FontWeight::ExtraBold => Weight::EXTRA_BOLD,
                    FontWeight::Black => Weight::BLACK,
                    FontWeight::ExtraBlack => Weight::EXTRA_BLACK,
                };

                text_style.set_font_style(FontStyle::new(weight, Width::NORMAL, Slant::Upright));
            }

            paragraph_style.set_text_style(&text_style);
        }

        let mut builder = ParagraphBuilder::new(&paragraph_style, &self.font_collection);
        builder.add_text(text);
        builder.build()
    }
}
