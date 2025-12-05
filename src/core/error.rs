// ─────────────────────────────────────────────────────────────────────────────
// SPELL - Error Types
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

//! SPELL error types with explicit type error support.

use std::fmt;
use super::types::SpellType;

#[derive(Debug, Clone)]
pub enum Error {
    /// Node not found in graph
    NodeNotFound(String),
    
    /// Cycle detected in dataflow graph
    CycleDetected(String),
    
    /// Missing required input
    MissingInput { 
        node: String, 
        port: String 
    },
    
    /// Type mismatch - EXPLICIT type errors
    TypeMismatch { 
        node: String, 
        port: String,
        expected: SpellType, 
        actual: SpellType,
    },
    
    /// Value doesn't match declared type
    InvalidValue {
        node: String,
        port: String,
        expected_type: SpellType,
        actual_value: String,
    },
    
    /// Legacy: Invalid type (for backwards compatibility)
    InvalidType { 
        node: String, 
        expected: String, 
        actual: String 
    },
    
    /// Operation-specific error
    OperationError { 
        node: String, 
        reason: String 
    },
    
    /// Unknown operation
    UnknownOperation(String),
    
    /// Missing type annotation (when explicit types are required)
    MissingTypeAnnotation {
        node: String,
        port: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NodeNotFound(id) => 
                write!(f, "Node not found: '{}'", id),
            
            Error::CycleDetected(id) => 
                write!(f, "Cycle detected at node: '{}'", id),
            
            Error::MissingInput { node, port } => 
                write!(f, "Node '{}' missing required input: '{}'", node, port),
            
            Error::TypeMismatch { node, port, expected, actual } => 
                write!(f, "Type mismatch in node '{}' port '{}': expected {}, got {}", 
                       node, port, expected, actual),
            
            Error::InvalidValue { node, port, expected_type, actual_value } =>
                write!(f, "Invalid value in node '{}' port '{}': expected type {}, got value '{}'",
                       node, port, expected_type, actual_value),
            
            Error::InvalidType { node, expected, actual } => 
                write!(f, "Node '{}' expected type '{}', got '{}'", node, expected, actual),
            
            Error::OperationError { node, reason } => 
                write!(f, "Operation failed in node '{}': {}", node, reason),
            
            Error::UnknownOperation(op) => 
                write!(f, "Unknown operation: '{}'", op),
            
            Error::MissingTypeAnnotation { node, port } =>
                write!(f, "Missing type annotation in node '{}' port '{}' - SPELL requires explicit types", 
                       node, port),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
