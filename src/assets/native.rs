use std::path::{Component, Path, PathBuf};

pub fn load_resource(
    src: &str,
    allow_network_asset: bool,
    asset_root: &Option<PathBuf>,
) -> anyhow::Result<Vec<u8>> {
    let src = src.trim();

    if src.starts_with("http://") || src.starts_with("https://") {
        if !allow_network_asset {
            anyhow::bail!("network asset is disabled");
        }

        load_url(src)
    } else {
        let path = resolve_asset_path(src, asset_root);
        Ok(std::fs::read(path)?)
    }
}

fn load_url(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut response = ureq::get(url).call()?;
    Ok(response.body_mut().read_to_vec()?)
}

fn resolve_asset_path(path: &str, asset_root: &Option<PathBuf>) -> PathBuf {
    let path = Path::new(path);

    match asset_root {
        Some(root) => root.join(normalize_path(path)),
        None => path.to_path_buf(),
    }
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

