use skia_safe::svg::Dom;

use crate::render::{
    RenderNode, RenderNodeKind,
    args::Args,
    context::RenderContext,
    styles::{CommonStyle, SvgStyle},
};

pub fn svg_func(
    _name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    render_context: &RenderContext,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let svg_style = SvgStyle::take_from(&mut args)?;

    let src = args.take_required_string("src")?;
    args.finish()?;

    let svg = load_svg(&render_context, &src);
    if svg.is_none() {
        eprintln!("svg not loaded: {}", src);
    }

    Ok(RenderNode {
        kind: RenderNodeKind::Svg {
            style: svg_style,
            svg,
        },
        children,
        style: common_style,
    })
}

fn load_svg(render_context: &RenderContext, src: &str) -> Option<Dom> {
    let image = render_context.asset_provider.load(&src).ok()?;
    Dom::from_bytes(&image, render_context.font_manager.get_font_mgr()).ok()
}
