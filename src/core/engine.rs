// ─────────────────────────────────────────────────────────────────────────────
// SPELL - Execution Engine
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

//! SPELL execution engine.
//!
//! All types MUST be explicitly declared - no inference, no legacy support.

use super::schema::{Graph, Node};
use super::types::{SpellType, TypedValue};
use super::ops::Ops;
use super::error::{Error, Result};
use std::collections::{HashMap, HashSet};
use serde_json::Value;

/// SPELL execution engine.
pub struct Engine {
    graph: Graph,
    cache: HashMap<String, Value>,
    type_cache: HashMap<String, SpellType>,
}

impl Engine {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            cache: HashMap::new(),
            type_cache: HashMap::new(),
        }
    }

    /// Executes all nodes in the graph.
    pub fn run(&mut self) -> () {
        let node_ids: Vec<String> = self.graph.nodes.keys().cloned().collect();
        
        for node_id in node_ids {
            let mut visiting: HashSet<String> = HashSet::new();
            match self.execute_node(&node_id, &mut visiting) {
                Ok(_) => {},
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    fn execute_node(&mut self, node_id: &str, visiting: &mut HashSet<String>) -> Result<Value> {
        // 1. Check Cache
        if let Some(cached) = self.cache.get(node_id) {
            return Ok(cached.clone());
        }

        // 2. Cycle Detection
        if visiting.contains(node_id) {
            return Err(Error::CycleDetected(node_id.to_string()));
        }
        let _: bool = visiting.insert(node_id.to_string());

        // 3. Get Node Definition
        let node: Node = self.graph.nodes.get(node_id)
            .ok_or_else(|| Error::NodeNotFound(node_id.to_string()))?
            .clone();

        // 4. Resolve Arguments
        let mut resolved_args: HashMap<String, Value> = HashMap::new();
        let typed_args_results: HashMap<String, Result<TypedValue>> = node.get_all_typed_args();
        
        for (key, typed_result) in typed_args_results {
            let typed_value: TypedValue = typed_result.map_err(|e: Error| -> Error {
                match e {
                    Error::MissingTypeAnnotation { port, .. } => 
                        Error::MissingTypeAnnotation { node: node_id.to_string(), port },
                    _ => e,
                }
            })?;
            let resolved: Value = self.resolve_typed_value(&typed_value, node_id, &key, visiting)?;
            let _: Option<Value> = resolved_args.insert(key, resolved);
        }

        // 5. Execute Operation
        let op: Box<dyn super::ops::Operation> = Ops::get(&node.op)
            .ok_or_else(|| Error::UnknownOperation(node.op.clone()))?;
        
        let result: HashMap<String, Value> = op.execute(&resolved_args)
            .map_err(|e: Error| -> Error { 
                match e {
                    Error::MissingInput { port, .. } => 
                        Error::MissingInput { node: node_id.to_string(), port },
                    Error::InvalidType { expected, actual, .. } => 
                        Error::InvalidType { node: node_id.to_string(), expected, actual },
                    Error::OperationError { reason, .. } => 
                        Error::OperationError { node: node_id.to_string(), reason },
                    _ => e,
                }
            })?;

        // 6. Type Check Output
        if let Some(ref declared_type) = node.returns {
            if let Some(out_val) = result.get("out") {
                if !declared_type.matches(out_val) {
                    return Err(Error::InvalidValue {
                        node: node_id.to_string(),
                        port: "out".to_string(),
                        expected_type: declared_type.clone(),
                        actual_value: format!("{}", out_val),
                    });
                }
                let _: Option<SpellType> = self.type_cache.insert(
                    node_id.to_string(), 
                    declared_type.clone()
                );
            }
        }

        // 7. Cache Results
        if let Some(out_val) = result.get("out") {
            let _: Option<Value> = self.cache.insert(node_id.to_string(), out_val.clone());
        }
        for (port, val) in &result {
            if port != "out" {
                let key: String = format!("{}:{}", node_id, port);
                let _: Option<Value> = self.cache.insert(key, val.clone());
            }
        }

        let _: bool = visiting.remove(node_id);

        result.get("out").cloned().ok_or_else(|| Error::OperationError {
            node: node_id.to_string(),
            reason: "Operation produced no 'out' output".to_string(),
        })
    }

    /// Resolves a typed value. REQUIRES explicit types.
    fn resolve_typed_value(
        &mut self, 
        typed_value: &TypedValue, 
        node_id: &str,
        port_name: &str,
        visiting: &mut HashSet<String>
    ) -> Result<Value> {
        // Check if value has explicit type
        let declared_type: &SpellType = typed_value.get_type()
            .ok_or_else(|| Error::MissingTypeAnnotation {
                node: node_id.to_string(),
                port: port_name.to_string(),
            })?;

        if typed_value.is_reference() {
            // Typed Reference
            let reference: &str = typed_value.get_reference()
                .ok_or_else(|| Error::OperationError {
                    node: node_id.to_string(),
                    reason: "Invalid reference".to_string(),
                })?;
            
            // Execute the referenced node
            let resolved: Value = self.execute_node(reference, visiting)?;
            
            // Type check
            if !declared_type.matches(&resolved) {
                let actual_type: SpellType = self.type_cache.get(reference)
                    .cloned()
                    .unwrap_or(SpellType::Any);
                
                return Err(Error::TypeMismatch {
                    node: node_id.to_string(),
                    port: port_name.to_string(),
                    expected: declared_type.clone(),
                    actual: actual_type,
                });
            }
            
            Ok(resolved)
        } else if let Some(literal) = typed_value.get_literal() {
            // Typed Literal
            if !declared_type.matches(literal) {
                return Err(Error::InvalidValue {
                    node: node_id.to_string(),
                    port: port_name.to_string(),
                    expected_type: declared_type.clone(),
                    actual_value: format!("{}", literal),
                });
            }
            Ok(literal.clone())
        } else {
            Err(Error::MissingTypeAnnotation {
                node: node_id.to_string(),
                port: port_name.to_string(),
            })
        }
    }
}
