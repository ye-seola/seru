use crate::render::{
    RenderNode, RenderNodeKind,
    args::Args,
    context::RenderContext,
    styles::{CommonStyle, TextStyle},
};

pub fn text_func(
    _name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    _render_context: &RenderContext,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let text_style = TextStyle::take_from(&mut args)?;

    let text = args.take_required_string("text")?;
    args.finish()?;

    Ok(RenderNode {
        kind: RenderNodeKind::Text {
            style: text_style,
            text,
        },
        children,
        style: common_style,
    })
}
