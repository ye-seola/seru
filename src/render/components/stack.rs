use crate::render::{self, RenderNode, args::Args, context::RenderContext, styles::*};

pub fn stack_func(
    _name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    _render_context: &RenderContext,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let stack_style = StackStyle::take_from(&mut args)?;

    args.finish()?;

    Ok(RenderNode {
        kind: render::RenderNodeKind::Stack { style: stack_style },
        children,
        style: common_style,
    })
}
