use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)] // TODO: remove debug
pub struct VaultExportData {
    pub path: String,
    pub data: HashMap<String, String>, // TODO: think about secrets protection on JSON export
}

impl VaultExportData {
    pub fn new(path: String, data: HashMap<String, String>) -> Self {
        Self { path, data }
    }
}
