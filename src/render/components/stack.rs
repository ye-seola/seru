use crate::{
    assets::AssetProvider,
    render::{self, RenderNode, args::Args, styles::*},
};

pub fn stack_func(
    name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    _asset_provider: &dyn AssetProvider,
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
