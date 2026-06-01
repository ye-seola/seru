use std::collections::HashMap;

use anyhow::Context;

use crate::core::Value;

pub struct Args {
    inner: HashMap<String, Value>,
}

impl Args {
    pub fn new(inner: HashMap<String, Value>) -> Self {
        Self { inner }
    }

    pub fn take(&mut self, name: &str) -> Option<Value> {
        self.inner.remove(name)
    }

    pub fn take_required(&mut self, name: &str) -> anyhow::Result<Value> {
        self.take(name)
            .with_context(|| format!("missing required argument: {}", name))
    }

    pub fn take_string(&mut self, name: &str) -> anyhow::Result<Option<String>> {
        self.take(name).map(|v| v.into_string()).transpose()
    }

    pub fn take_required_string(&mut self, name: &str) -> anyhow::Result<String> {
        self.take_required(name)?.into_string()
    }

    pub fn take_number(&mut self, name: &str) -> anyhow::Result<Option<f32>> {
        self.take(name).map(|v| v.into_number()).transpose()
    }

    pub fn finish(self) -> anyhow::Result<()> {
        if let Some(name) = self.inner.keys().next() {
            anyhow::bail!("unknown argument: {}", name);
        }

        Ok(())
    }
}
