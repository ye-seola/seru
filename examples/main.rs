use std::{collections::HashMap, fs};

use seru::{
    Seru, SeruOption,
    render::{RenderOptions, RenderOutputType, styles::Color},
};

fn main() -> anyhow::Result<()> {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    println!("asset_root: {}", asset_root.display());

    let src = include_str!("simple0.seru");

    let mut seru = Seru::new_with_options(&SeruOption {
        allow_network_asset: true,
        asset_root: Some(asset_root),
        load_system_fonts: true,
        fonts: vec![],
    })?;
    seru.load_str(src)?;

    let img = seru.render(
        "Main",
        HashMap::new(),
        RenderOptions {
            output_type: RenderOutputType::PNG,
            background: Some(Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            }),
            render_scale: Some(2.0),
            width: 320.0,
            height: 420.0,
        },
    )?;

    fs::write("sample0.png", img)?;
    Ok(())
}
