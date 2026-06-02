use crate::render::{self, RenderNode, args::Args, context::RenderContext, styles::*};

pub fn box_func(
    name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    _render_context: &RenderContext,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let box_style = BoxStyle::take_from(name, &mut args)?;

    args.finish()?;

    Ok(RenderNode {
        kind: render::RenderNodeKind::Box { style: box_style },
        children,
        style: common_style,
    })
}
