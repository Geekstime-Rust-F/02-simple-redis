use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;

use crate::RespFrame;

#[derive(Debug)]
pub struct Backend(Arc<BackendInner>);

impl Backend {
    pub fn new() -> Self {
        Self(Arc::new(BackendInner::new()))
    }
}

#[derive(Debug)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
}

impl BackendInner {
    fn new() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
        }
    }
}

impl BackendInner {
    fn default() -> BackendInner {
        Self::new()
    }
}

impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Backend {
    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: &str, value: RespFrame) {
        self.map.insert(key.to_string(), value);
    }

    pub fn hsget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: &str, field: &str, value: RespFrame) {
        let hmap = self
            .hmap
            .entry(key.to_string())
            .or_default();
        hmap.insert(field.to_string(), value);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.value().clone())
    }
}
