use skia_safe::{
    Canvas, Color as SKColor, Paint, PaintStyle, Point, RRect, Rect, canvas::SrcRectConstraint,
    surfaces, svg::Canvas as SvgCanvas,
};

use crate::{
    font::FontManager,
    layout::LayoutNode,
    render::{
        RenderNodeKind, RenderOptions, RenderOutputType,
        styles::{Color, ImageFit},
    },
};

pub fn render(
    options: &RenderOptions,
    layout_tree: &LayoutNode<'_>,
    font_manager: &FontManager,
) -> anyhow::Result<Vec<u8>> {
    let bounds = Rect::from_xywh(0.0, 0.0, options.width, options.height);

    match options.output_type {
        RenderOutputType::PNG => {
            let scale = options.render_scale.unwrap_or(1.0);
            if scale <= 0.0 {
                anyhow::bail!("render_scale must be positive");
            }

            let mut surface = surfaces::raster_n32_premul((
                (options.width * scale).ceil() as i32,
                (options.height * scale).ceil() as i32,
            ))
            .ok_or_else(|| anyhow::anyhow!("failed to create surface"))?;

            let canvas = surface.canvas();
            canvas.scale((scale, scale));

            draw_background(canvas, bounds, &options.background)?;
            render_skia_node(layout_tree, 0.0, 0.0, canvas, &font_manager)?;

            let image = surface.image_snapshot();

            let data = image
                .encode(None, skia_safe::EncodedImageFormat::PNG, None)
                .ok_or_else(|| anyhow::anyhow!("failed to encode png"))?;

            Ok(data.as_bytes().to_vec())
        }

        RenderOutputType::SVG => {
            let canvas = SvgCanvas::new(bounds, None);

            draw_background(&canvas, bounds, &options.background)?;
            render_skia_node(layout_tree, 0.0, 0.0, &canvas, &font_manager)?;

            let data = canvas.end();
            Ok(data.to_vec())
        }
    }
}

fn render_skia_node(
    node: &LayoutNode,
    offset_x: f32,
    offset_y: f32,
    canvas: &Canvas,
    font_manager: &FontManager,
) -> anyhow::Result<()> {
    let x = (offset_x + node.layout.x).round();
    let y = (offset_y + node.layout.y).round();
    let w = (node.layout.width).round();
    let h = (node.layout.height).round();

    let radius = node.render.style.radius.unwrap_or(0.0);

    let rect = Rect::from_xywh(x, y, w, h);
    let rrect = RRect::new_rect_xy(rect, radius, radius);

    {
        canvas.save();
        if radius > 0.0 {
            canvas.clip_rrect(rrect, None, Some(true));
        } else {
            canvas.clip_rect(rect, None, Some(false));
        }
    }

    // basic paint
    if let Some(bg) = &node.render.style.background {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(*bg);
        canvas.draw_rrect(rrect, &paint);
    }

    if let (Some(border_width), Some(border_color)) =
        (node.render.style.border, node.render.style.border_color)
    {
        if border_width > 0.0 {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(border_width);
            paint.set_color(border_color);

            canvas.draw_rrect(rrect, &paint);
        }
    }

    // draw text, image
    match &node.render.kind {
        RenderNodeKind::Text { style, text } => {
            let mut paragraph = font_manager.create_paragraph(style, text);
            paragraph.layout(w);
            paragraph.paint(canvas, Point { x, y });
        }
        RenderNodeKind::Image {
            style,
            image: Some(image),
        } => {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            let fit = style.fit.unwrap_or(ImageFit::Fill);

            let (src_rect, dst_rect) = image_fit_rects(image, rect, fit);
            let src_rect = src_rect
                .as_ref()
                .map(|rect| (rect, SrcRectConstraint::Strict));

            canvas.draw_image_rect(image, src_rect, dst_rect, &paint);
        }
        _ => {}
    }

    for child in &node.children {
        render_skia_node(child, x, y, canvas, font_manager)?;
    }

    {
        canvas.restore();
    }

    Ok(())
}

fn image_fit_rects(image: &skia_safe::Image, bounds: Rect, fit: ImageFit) -> (Option<Rect>, Rect) {
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    if image_width <= 0.0 || image_height <= 0.0 {
        return (None, bounds);
    }

    match fit {
        ImageFit::Fill => (None, bounds),
        ImageFit::Cover => {
            let scale = (image_width / bounds.width()).min(image_height / bounds.height());
            let width = bounds.width() * scale;
            let height = bounds.height() * scale;
            let x = (image_width - width) / 2.0;
            let y = (image_height - height) / 2.0;
            let src = Rect::from_xywh(x, y, width, height);

            (Some(src), bounds)
        }
        ImageFit::Contain => {
            let scale = (bounds.width() / image_width).min(bounds.height() / image_height);
            let width = image_width * scale;
            let height = image_height * scale;
            let x = bounds.left + (bounds.width() - width) / 2.0;
            let y = bounds.top + (bounds.height() - height) / 2.0;

            (None, Rect::from_xywh(x, y, width, height))
        }
    }
}

fn draw_background(
    canvas: &Canvas,
    bounds: Rect,
    background: &Option<Color>,
) -> anyhow::Result<()> {
    let Some(background) = background else {
        return Ok(());
    };

    let mut paint = Paint::default();
    paint.set_color(*background);

    canvas.draw_rect(bounds, &paint);
    Ok(())
}

impl From<Color> for SKColor {
    fn from(color: Color) -> Self {
        SKColor::from_argb(color.a, color.r, color.g, color.b)
    }
}
