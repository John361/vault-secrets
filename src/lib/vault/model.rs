use std::collections::HashMap;

use crate::secret::Secret;

#[derive(Debug)] // TODO: remove debug
pub struct VaultExportData {
    pub path: String,
    pub data: HashMap<String, Secret>, // TODO: think about secrets protection on JSON export
}

impl VaultExportData {
    pub fn new(path: String, data: HashMap<String, Secret>) -> Self {
        Self { path, data }
    }
}
