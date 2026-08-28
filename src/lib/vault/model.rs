use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::secret::Secret;

#[derive(Deserialize, Serialize)]
pub struct VaultData {
    pub path: String,
    pub data: HashMap<String, Secret>,
    pub metadata: HashMap<String, Secret>,
}

impl VaultData {
    pub fn new(path: String, data: HashMap<String, Secret>, metadata: HashMap<String, Secret>) -> Self {
        Self { path, data, metadata }
    }
}
