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
            return load_url(src);
        } else {
            let path = resolve_asset_path(src, &self.asset_root)?;
            Ok(std::fs::read(path)?)
        }
    }
}

fn load_url(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut response = ureq::get(url).call()?;
    Ok(response.body_mut().read_to_vec()?)
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
