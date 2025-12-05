// ─────────────────────────────────────────────────────────────────────────────
// SPELL - Type System
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SpellType {
    Number,
    String,
    Boolean,
    Array(Box<SpellType>),
    Any,
    Unit,
}

impl SpellType {
    pub fn parse(s: &str) -> Result<SpellType, String> {
        let s: &str = s.trim();
        
        match s {
            "Number" => Ok(SpellType::Number),
            "String" => Ok(SpellType::String),
            "Boolean" => Ok(SpellType::Boolean),
            "Any" => Ok(SpellType::Any),
            "Unit" => Ok(SpellType::Unit),
            _ if s.starts_with("Array<") && s.ends_with('>') => {
                let inner: &str = &s[6..s.len()-1];
                let inner_type: SpellType = SpellType::parse(inner)?;
                Ok(SpellType::Array(Box::new(inner_type)))
            }
            _ => Err(format!("Unknown type: '{}'", s)),
        }
    }

    pub fn matches(&self, value: &serde_json::Value) -> bool {
        match (self, value) {
            (SpellType::Number, serde_json::Value::Number(_)) => true,
            (SpellType::String, serde_json::Value::String(_)) => true,
            (SpellType::Boolean, serde_json::Value::Bool(_)) => true,
            (SpellType::Unit, serde_json::Value::Null) => true,
            (SpellType::Any, _) => true,
            (SpellType::Array(inner), serde_json::Value::Array(arr)) => {
                arr.iter().all(|item: &serde_json::Value| inner.matches(item))
            }
            _ => false,
        }
    }
}

impl fmt::Display for SpellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpellType::Number => write!(f, "Number"),
            SpellType::String => write!(f, "String"),
            SpellType::Boolean => write!(f, "Boolean"),
            SpellType::Any => write!(f, "Any"),
            SpellType::Unit => write!(f, "Unit"),
            SpellType::Array(inner) => write!(f, "Array<{}>", inner),
        }
    }
}

impl TryFrom<String> for SpellType {
    type Error = String;
    
    fn try_from(s: String) -> Result<Self, Self::Error> {
        SpellType::parse(&s)
    }
}

impl From<SpellType> for String {
    fn from(t: SpellType) -> String {
        t.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypedValue {
    Reference {
        #[serde(rename = "ref")]
        reference: String,
        #[serde(rename = "type")]
        value_type: SpellType,
    },
    Literal {
        literal: serde_json::Value,
        #[serde(rename = "type")]
        value_type: SpellType,
    },
}

impl TypedValue {
    pub fn get_type(&self) -> Option<&SpellType> {
        match self {
            TypedValue::Reference { value_type, .. } => Some(value_type),
            TypedValue::Literal { value_type, .. } => Some(value_type),
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, TypedValue::Reference { .. })
    }

    pub fn get_reference(&self) -> Option<&str> {
        match self {
            TypedValue::Reference { reference, .. } => Some(reference),
            _ => None,
        }
    }

    pub fn get_literal(&self) -> Option<&serde_json::Value> {
        match self {
            TypedValue::Literal { literal, .. } => Some(literal),
            _ => None,
        }
    }
}
