use std::ffi::CString;
use std::os::raw::c_char;
use std::path::PathBuf;

#[repr(C)]
pub struct Bytes {
    pub len: u32,
    pub ptr: *mut u8,
}

unsafe extern "C" {
    fn js_fetch(url: *const c_char) -> *mut Bytes;
}

pub fn load_resource(
    src: &str,
    allow_network_asset: bool,
    _asset_root: &Option<PathBuf>,
) -> anyhow::Result<Vec<u8>> {
    if !allow_network_asset {
        anyhow::bail!("network asset is disabled");
    }

    request(src.trim())
}

fn request(url: &str) -> anyhow::Result<Vec<u8>> {
    let url = CString::new(url)?;

    let raw = unsafe { js_fetch(url.as_ptr()) };
    if raw.is_null() {
        anyhow::bail!("js_fetch returned null");
    }

    let bytes = unsafe { Box::from_raw(raw) };

    let data = unsafe { Vec::from_raw_parts(bytes.ptr, bytes.len as usize, bytes.len as usize) };

    Ok(data)
}
