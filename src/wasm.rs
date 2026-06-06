#[cfg(target_os = "emscripten")]
mod wasm_impl {
    use anyhow::Context;
    use seru::font::FontManager;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use seru::{
        assets::DefaultAssetProvider,
        core::Value,
        font::{Font, FontFile},
        render::{RenderOptions, RenderOutputType, styles::Color},
    };

    thread_local! {
        static FONT_MANAGER: RefCell<Option<Rc<FontManager>>> =
            const { RefCell::new(None) };

        static ASSET_PROVIDER: RefCell<Option<Rc<DefaultAssetProvider>>> =
            const { RefCell::new(None) };
    }

    #[repr(C)]
    pub struct Bytes {
        pub len: u32,
        pub ptr: *mut u8,
    }

    #[repr(C)]
    pub struct ResultBytes {
        pub success: u32,
        pub len: u32,
        pub ptr: *mut u8,
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn render_png(
        width: u32,
        height: u32,
        source: *mut Bytes,
        args: *mut Bytes,
    ) -> *mut ResultBytes {
        match render_png_inner(width, height, source, args) {
            Ok(data) => into_result_bytes(1, data),
            Err(err) => into_result_bytes(0, err.to_string().into_bytes()),
        }
    }

    fn into_result_bytes(success: u32, mut data: Vec<u8>) -> *mut ResultBytes {
        let len = data.len() as u32;
        let ptr = data.as_mut_ptr();

        std::mem::forget(data);

        Box::into_raw(Box::new(ResultBytes { success, len, ptr }))
    }

    fn render_png_inner(
        width: u32,
        height: u32,
        source: *mut Bytes,
        args: *mut Bytes,
    ) -> anyhow::Result<Vec<u8>> {
        let source = unsafe { take_bytes(source)? };
        let source = std::str::from_utf8(&source)?;

        let args = unsafe { take_bytes(args)? };
        let args = std::str::from_utf8(&args)?;

        let font_manager = get_font_manager()?;

        let asset_provider = get_asset_provider();

        let mut runtime = seru::runtime::Runtime::new(seru::render::context::RenderContext {
            font_manager: font_manager.clone(),
            asset_provider,
        });
        runtime.register_builtin_functions();

        let prog = seru::language::parser::Parser::from_src(source)?.parse()?;
        runtime.evaluate(&prog)?;

        let args = json_to_args(serde_json::from_str(args)?)?;

        let node = runtime
            .build_render_tree("Main", args)?
            .context("component returned no render node")?;

        let layout =
            seru::layout::build_layout_tree(&node, width as usize, height as usize, &font_manager)?;

        let options = RenderOptions {
            output_type: RenderOutputType::PNG,
            background: Some(Color {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF,
                a: 0xFF,
            }),
            render_scale: Some(2.0),
            width: width as f32,
            height: height as f32,
        };
        seru::render::render(&options, &layout, &font_manager)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn alloc(len: usize) -> *mut Bytes {
        let mut data = Vec::<u8>::with_capacity(len);
        let ptr = data.as_mut_ptr();

        std::mem::forget(data);

        Box::into_raw(Box::new(Bytes {
            len: len as u32,
            ptr,
        }))
    }

    fn json_to_args(json: serde_json::Value) -> anyhow::Result<HashMap<String, Value>> {
        match json {
            serde_json::Value::Object(map) => Ok(map
                .into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect()),
            _ => anyhow::bail!("expected JSON object!!"),
        }
    }

    fn json_to_value(value: serde_json::Value) -> Value {
        match value {
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Number(v) => Value::Number(v.as_f64().unwrap_or(0.0) as f32),
            serde_json::Value::Array(v) => Value::Array(v.into_iter().map(json_to_value).collect()),
            serde_json::Value::Object(v) => {
                Value::Dict(v.into_iter().map(|(k, v)| (k, json_to_value(v))).collect())
            }
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Null => Value::Null,
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn free_result_bytes(ptr: *mut ResultBytes) {
        if ptr.is_null() {
            return;
        }

        unsafe {
            let result = Box::from_raw(ptr);

            if !result.ptr.is_null() {
                drop(Vec::from_raw_parts(
                    result.ptr,
                    result.len as usize,
                    result.len as usize,
                ));
            }
        }
    }

    unsafe fn take_bytes(ptr: *mut Bytes) -> anyhow::Result<Vec<u8>> {
        if ptr.is_null() {
            anyhow::bail!("null Bytes ptr");
        }

        let bytes = unsafe { Box::from_raw(ptr) };

        Ok(unsafe { Vec::from_raw_parts(bytes.ptr, bytes.len as usize, bytes.len as usize) })
    }

    fn get_font_manager() -> anyhow::Result<Rc<FontManager>> {
        FONT_MANAGER.with(|cell| {
            if let Some(manager) = cell.borrow().as_ref() {
                return Ok(manager.clone());
            }

            let manager = Rc::new(FontManager::new(
                false,
                &[FontFile {
                    font: Font::Bytes(include_bytes!("./PretendardJP-Regular.ttf").to_vec()),
                    alias: Some("Pretendard".to_string()),
                }],
            )?);

            *cell.borrow_mut() = Some(manager.clone());

            Ok(manager)
        })
    }

    fn get_asset_provider() -> Rc<DefaultAssetProvider> {
        ASSET_PROVIDER.with(|cell| {
            if let Some(provider) = cell.borrow().as_ref() {
                return provider.clone();
            }

            let provider = Rc::new(DefaultAssetProvider {
                allow_network_asset: true,
                asset_root: None,
            });

            *cell.borrow_mut() = Some(provider.clone());

            provider
        })
    }
}

#[cfg(target_os = "emscripten")]
fn main() {}

#[cfg(not(target_os = "emscripten"))]
fn main() {
    panic!("this binary is only for emscripten");
}
