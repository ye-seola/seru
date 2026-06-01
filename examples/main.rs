use std::{collections::HashMap, env, fs};

use seru::{
    Seru, SeruOption,
    render::{RenderOptions, RenderOutputType, styles::Color},
};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    println!("asset_root: {}", asset_root.display());

    let mut width = 500.0;
    let mut height = 500.0;

    let seru_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "simple0.seru".to_string()
    };

    if args.len() > 2 {
        width = args[2].parse()?;
    }

    if args.len() > 3 {
        height = args[3].parse()?;
    }

    let source = fs::read_to_string(asset_root.join(seru_path))?;

    let mut seru = Seru::new_with_options(&SeruOption {
        allow_network_asset: true,
        asset_root: Some(asset_root),
        load_system_fonts: true,
        fonts: vec![],
    })?;
    seru.load_str(&source)?;

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
            width: width,
            height: height,
        },
    )?;

    fs::write("sample0.png", img)?;
    Ok(())
}
