use skia_safe::{Data, Image};

use crate::{
    assets::AssetProvider,
    core::{ImageWrap, Value},
    render::{self, RenderNode, args::Args, styles::*},
};

pub fn image_func(
    _name: &str,
    mut args: Args,
    children: Vec<RenderNode>,
    asset_provider: &dyn AssetProvider,
) -> anyhow::Result<RenderNode> {
    let common_style = CommonStyle::take_from(&mut args)?;
    let image_style = ImageStyle::take_from(&mut args)?;

    let src = args.take_required("src")?;
    args.finish()?;

    let image = match src {
        Value::String(src) => {
            let image = load_image(asset_provider, &src);
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

fn load_image(asset_provider: &dyn AssetProvider, src: &str) -> Option<Image> {
    let image = asset_provider.load(&src).ok()?;
    Image::from_encoded(Data::new_copy(&image))
}
