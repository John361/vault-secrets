use std::collections::HashMap;

use crate::secret::Secret;

#[derive(Debug)] // TODO: remove debug
pub struct VaultExportData {
    path: String,
    data: HashMap<String, Secret>, // TODO: think about secrets protection on JSON export
}

impl VaultExportData {
    pub fn new(path: String, data: HashMap<String, Secret>) -> Self {
        Self { path, data }
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn data(&self) -> &HashMap<String, Secret> {
        &self.data
    }
}
