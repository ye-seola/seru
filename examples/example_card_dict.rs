use std::{collections::HashMap, fs};

use seru::{
    Seru, SeruOption,
    core::Value,
    render::{RenderOptions, RenderOutputType, styles::Color},
};

fn main() -> anyhow::Result<()> {
    let width = 500.0;
    let height = 500.0;

    let seru = Seru::new_with_options(&SeruOption {
        allow_network_asset: true,
        asset_root: None,
        load_system_fonts: true,
        fonts: vec![],
    })?;
    
    let mut template = seru.compile_str(include_str!("example_card_dict.seru"))?;
    let img = template.render(
        "Main",
        HashMap::from([(
            "data".to_string(),
            Value::Array(vec![
                Value::Dict(HashMap::from([(
                    "name".to_string(),
                    Value::String("Alice".to_string()),
                )])),
                Value::Dict(HashMap::from([(
                    "name".to_string(),
                    Value::String("Bob".to_string()),
                )])),
            ]),
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

    fs::write("sample4.png", img)?;
    Ok(())
}
