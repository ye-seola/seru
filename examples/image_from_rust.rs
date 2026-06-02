use std::{collections::HashMap, fs};

use seru::{
    Seru, SeruOption,
    render::{RenderOptions, RenderOutputType, styles::Color},
};

fn main() -> anyhow::Result<()> {
    let width = 500.0;
    let height = 500.0;

    let mut seru = Seru::new_with_options(&SeruOption {
        allow_network_asset: true,
        asset_root: None,
        load_system_fonts: true,
        fonts: vec![],
    })?;

    let image_data = include_bytes!("img1.png");
    seru.load_str(
        r#"
component Main(from_rust):
    Image(src=from_rust, fit="cover", width="100%", height="100%")
    "#,
    )?;

    let img = seru.render(
        "Main",
        HashMap::from([(
            "from_rust".to_string(),
            seru.load_image(image_data).expect("failed to load image"),
        )]),
        RenderOptions {
            output_type: RenderOutputType::PNG,
            background: Some(Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            }),
            render_scale: Some(2.0),
            width: width,
            height: height,
        },
    )?;

    fs::write("image_from_rust.png", img)?;
    Ok(())
}
