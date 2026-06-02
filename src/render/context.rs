use std::rc::Rc;

use crate::{assets::AssetProvider, font::FontManager};

pub struct RenderContext {
    pub font_manager: Rc<FontManager>,
    pub asset_provider: Rc<dyn AssetProvider>,
}
