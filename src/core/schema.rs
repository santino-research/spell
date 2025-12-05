// ─────────────────────────────────────────────────────────────────────────────
// SPELL - Schema
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{SpellType, TypedValue};
use super::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    #[serde(flatten)]
    pub nodes: HashMap<String, Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub op: String,

    #[serde(default)]
    pub returns: Option<SpellType>,

    #[serde(flatten)]
    pub args: HashMap<String, serde_json::Value>,
}

impl Node {
    pub fn get_all_typed_args(&self) -> HashMap<String, Result<TypedValue>> {
        let mut result: HashMap<String, Result<TypedValue>> = HashMap::new();
        
        for (key, value) in &self.args {
            if key == "op" || key == "returns" {
                continue;
            }
            
            let typed: Result<TypedValue> = serde_json::from_value::<TypedValue>(value.clone())
                .map_err(|_| Error::MissingTypeAnnotation {
                    node: "".to_string(),
                    port: key.clone(),
                });
            
            let _: Option<Result<TypedValue>> = result.insert(key.clone(), typed);
        }
        
        result
    }
}
