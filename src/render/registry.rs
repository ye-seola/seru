use std::collections::HashMap;

use anyhow::Context;

use crate::{
    core::Value,
    render::{RenderNode, args::Args, components, context::RenderContext},
};

pub type ComponentFunc = fn(
    name: &str,
    args: Args,
    children: Vec<RenderNode>,
    render_context: &RenderContext,
) -> anyhow::Result<RenderNode>;

#[derive(Debug)]
pub struct NativeComponent {
    name: String,
    func: ComponentFunc,
}

impl NativeComponent {
    pub fn new(name: impl Into<String>, func: ComponentFunc) -> Self {
        Self {
            name: name.into(),
            func,
        }
    }
}

pub struct NativeComponentRegistry {
    components: HashMap<String, NativeComponent>,
    render_context: RenderContext,
}

impl NativeComponentRegistry {
    pub fn new(render_context: RenderContext) -> NativeComponentRegistry {
        let mut registry = NativeComponentRegistry {
            components: HashMap::new(),
            render_context,
        };

        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        let components = vec![
            NativeComponent::new("Box", components::box_func),
            NativeComponent::new("Column", components::box_func),
            NativeComponent::new("Center", components::box_func),
            NativeComponent::new("Row", components::box_func),
            NativeComponent::new("Stack", components::stack_func),
            NativeComponent::new("Text", components::text_func),
            NativeComponent::new("Image", components::image_func),
            NativeComponent::new("Svg", components::svg_func),
        ];

        for ele in components {
            self.register(ele).expect("dup");
        }
    }

    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    pub fn register(&mut self, component: NativeComponent) -> anyhow::Result<()> {
        if self.components.contains_key(&component.name) {
            anyhow::bail!("already registered component: {}", component.name);
        }

        self.components.insert(component.name.clone(), component);

        Ok(())
    }

    pub fn create(
        &self,
        name: &str,
        args: HashMap<String, Value>,
        children: Vec<RenderNode>,
    ) -> anyhow::Result<RenderNode> {
        let component = self
            .components
            .get(name)
            .with_context(|| format!("cannot find native component: {}", name))?;

        (component.func)(name, Args::new(args), children, &self.render_context)
    }
}
