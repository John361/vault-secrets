use std::collections::HashMap;

use crate::secret::Secret;

#[derive(Debug)] // TODO: remove debug
pub struct VaultExportData {
    pub path: String,
    pub data: HashMap<String, Secret>,
}

impl VaultExportData {
    pub fn new(path: String, data: HashMap<String, Secret>) -> Self {
        Self { path, data }
    }
}
