use std::rc::Rc;

use skia_safe::{Data, Image};

use crate::{
    assets::AssetProvider,
    core::{ImageWrap, Value},
    render::{self, RenderNode, args::Args, context::RenderContext, styles::*},
};

pub fn image_func(
    _name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    render_context: &RenderContext,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let image_style = ImageStyle::take_from(&mut args)?;

    let src = args.take_required("src")?;
    args.finish()?;

    let image = match src {
        Value::String(src) => {
            let image = load_image(&render_context.asset_provider, &src);
            if image.is_none() {
                eprintln!("image not loaded: {}", src);
            }

            image
        }
        Value::Image(ImageWrap(image)) => Some(image),
        _ => anyhow::bail!(""),
    };

    Ok(RenderNode {
        kind: render::RenderNodeKind::Image {
            style: image_style,
            image,
        },
        children,
        style: common_style,
    })
}

fn load_image(asset_provider: &Rc<dyn AssetProvider>, src: &str) -> Option<Image> {
    let image = asset_provider.load(&src).ok()?;
    Image::from_encoded(Data::new_copy(&image))
}
