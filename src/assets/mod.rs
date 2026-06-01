use std::path::{Component, Path, PathBuf};

pub trait AssetProvider {
    fn load(&self, src: &str) -> anyhow::Result<Vec<u8>>;
}

pub struct DefaultAssetProvider {
    pub allow_network_asset: bool,
    pub asset_root: Option<PathBuf>,
}

impl AssetProvider for DefaultAssetProvider {
    fn load(&self, src: &str) -> anyhow::Result<Vec<u8>> {
        let src = src.trim();
        if src.starts_with("http://") || src.starts_with("https://") {
            if !self.allow_network_asset {
                anyhow::bail!("allow_network_asset is disabled")
            }

            let mut response = ureq::get(src).call()?;
            return Ok(response
                .body_mut()
                .with_config()
                .limit(20 * 1024 * 1024)
                .read_to_vec()?);
        } else {
            let path = resolve_asset_path(src, &self.asset_root)?;
            Ok(std::fs::read(path)?)
        }
    }
}

fn resolve_asset_path(path: &str, asset_root: &Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = Path::new(path);
    let Some(asset_root) = asset_root else {
        return Ok(path.to_path_buf());
    };

    Ok(asset_root.join(normalize_path(path)))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }

    normalized
}
