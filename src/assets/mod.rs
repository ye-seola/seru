use std::path::PathBuf;

pub trait AssetProvider {
    fn load(&self, src: &str) -> anyhow::Result<Vec<u8>>;
}

pub struct DefaultAssetProvider {
    pub allow_network_asset: bool,
    pub asset_root: Option<PathBuf>,
}

impl AssetProvider for DefaultAssetProvider {
    fn load(&self, src: &str) -> anyhow::Result<Vec<u8>> {
        load_resource(src, self.allow_network_asset, &self.asset_root)
    }
}
#[cfg(target_os = "emscripten")]
mod emscripten;

#[cfg(not(target_os = "emscripten"))]
mod native;

#[cfg(target_os = "emscripten")]
use emscripten::load_resource;

#[cfg(not(target_os = "emscripten"))]
use native::load_resource;
