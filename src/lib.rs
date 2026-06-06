pub mod assets;
pub mod core;
pub mod font;
pub mod language;
pub mod layout;
pub mod render;
pub mod runtime;

use std::{collections::HashMap, path::PathBuf, rc::Rc};

use anyhow::Context;

use crate::{
    assets::DefaultAssetProvider,
    core::Value,
    font::{FontFile, FontManager},
    language::parser::Parser,
    layout::build_layout_tree,
    render::{RenderOptions, context::RenderContext},
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
    render_context: RenderContext,
    font_manager: Rc<FontManager>,
}

pub struct SeruTemplate {
    runtime: Runtime,
    font_manager: Rc<FontManager>,
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
        let font_manager = Rc::new(FontManager::new(options.load_system_fonts, &options.fonts)?);

        let asset_provider = Rc::new(DefaultAssetProvider {
            allow_network_asset: options.allow_network_asset,
            asset_root: options.asset_root.clone(),
        });

        Ok(Self {
            render_context: RenderContext {
                font_manager: font_manager.clone(),
                asset_provider,
            },
            font_manager,
        })
    }

    pub fn compile_str(&self, src: &str) -> anyhow::Result<SeruTemplate> {
        let prog = Parser::from_src(src)?.parse()?;

        let mut runtime = Runtime::new(self.render_context.clone());
        runtime.register_builtin_functions();
        runtime.evaluate(&prog)?;

        Ok(SeruTemplate {
            runtime,
            font_manager: self.font_manager.clone(),
        })
    }
}

impl SeruTemplate {
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
            &self.font_manager,
        )?;

        render::render(&options, &layout, &self.font_manager)
    }
}
