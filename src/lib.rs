pub mod assets;
pub mod core;
pub mod font;
pub mod language;
pub mod layout;
pub mod render;
pub mod runtime;

use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;

use crate::{
    assets::DefaultAssetProvider,
    core::Value,
    font::{FontFile, FontManager},
    language::parser::Parser,
    layout::build_layout_tree,
    render::RenderOptions,
    runtime::Runtime,
};

#[derive(Debug)]
pub struct SeruOption {
    pub allow_network_asset: bool,
    pub asset_root: Option<PathBuf>,
    pub load_system_fonts: bool,
    pub fonts: Vec<FontFile>,
}

pub struct Seru {
    runtime: Runtime,
    font_mgr: FontManager,
}

impl Seru {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self::new_with_options(&SeruOption {
            allow_network_asset: true,
            asset_root: None,
            load_system_fonts: true,
            fonts: vec![],
        })?)
    }

    pub fn new_with_options(options: &SeruOption) -> anyhow::Result<Self> {
        let asset_provider = Box::new(DefaultAssetProvider {
            allow_network_asset: options.allow_network_asset,
            asset_root: options.asset_root.clone(),
        });

        let mut runtime = Runtime::new(asset_provider);
        runtime.register_builtin_functions();

        Ok(Self {
            runtime,
            font_mgr: FontManager::new(options.load_system_fonts, &options.fonts)?,
        })
    }

    pub fn load_str(&mut self, src: &str) -> anyhow::Result<()> {
        let prog = Parser::from_src(src)?.parse()?;
        self.runtime.evaluate(&prog)?;
        Ok(())
    }

    pub fn render(
        &mut self,
        component: &str,
        args: HashMap<String, Value>,
        options: RenderOptions,
    ) -> anyhow::Result<Vec<u8>> {
        let node = self
            .runtime
            .build_render_tree(component, args)?
            .context("component returned no render node")?;

        let layout = build_layout_tree(
            &node,
            options.width as usize,
            options.height as usize,
            &self.font_mgr,
        )?;

        render::render(&options, &layout, &self.font_mgr)
    }
}
