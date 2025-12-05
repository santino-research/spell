// ─────────────────────────────────────────────────────────────────────────────
// SPELL
// Copyright (c) 2025 Santino Research. MIT License.
// ─────────────────────────────────────────────────────────────────────────────

//! SPELL operation implementations.
//!
//! Each operation implements the `Operation` trait.
//! Operations are stateless and thread-safe.

use serde_json::Value;
use std::collections::HashMap;
use super::error::{Error, Result};

/// Interface for all SPELL operations.
/// Operations must be stateless, thread-safe, and robust.
pub trait Operation: Send + Sync {
    /// Executes the operation with the given inputs.
    fn execute(
        &self,
        inputs: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>>;
}

/// Registry for operations.
pub struct Ops;

impl Ops {
    /// Creates an operation instance by name.
    pub fn get(op_name: &str) -> Option<Box<dyn Operation>> {
        match op_name {
            "Const" => Some(Box::new(ConstOp)),
            "Print" => Some(Box::new(PrintOp)),
            "Add" => Some(Box::new(MathOp::Add)),
            "Sub" => Some(Box::new(MathOp::Sub)),
            "Mul" => Some(Box::new(MathOp::Mul)),
            "Div" => Some(Box::new(MathOp::Div)),
            "Eq" => Some(Box::new(LogicOp::Eq)),
            "Gt" => Some(Box::new(LogicOp::Gt)),
            "Lt" => Some(Box::new(LogicOp::Lt)),
            "Switch" => Some(Box::new(SwitchOp)),
            "Map" => Some(Box::new(MapOp)),
            "Reduce" => Some(Box::new(ReduceOp)),
            "Len" => Some(Box::new(LenOp)),
            "Filter" => Some(Box::new(FilterOp)),
            _ => None,
        }
    }
}

// --- Helpers for Robust Input Extraction ---

fn get_input<'a>(inputs: &'a HashMap<String, Value>, name: &str) -> Result<&'a Value> {
    inputs.get(name).ok_or_else(|| Error::MissingInput {
        node: "unknown".to_string(), // Context will be added by engine
        port: name.to_string(),
    })
}

fn get_f64(inputs: &HashMap<String, Value>, name: &str) -> Result<f64> {
    let val = get_input(inputs, name)?;
    val.as_f64().ok_or_else(|| Error::InvalidType {
        node: "unknown".to_string(),
        expected: "number".to_string(),
        actual: format!("{:?}", val),
    })
}

fn get_bool(inputs: &HashMap<String, Value>, name: &str) -> Result<bool> {
    let val = get_input(inputs, name)?;
    val.as_bool().ok_or_else(|| Error::InvalidType {
        node: "unknown".to_string(),
        expected: "boolean".to_string(),
        actual: format!("{:?}", val),
    })
}

// ============================================================================
// OPERATION IMPLEMENTATIONS
// ============================================================================

/// Constant value operation.
/// Inputs: `value`
/// Outputs: `out`
struct ConstOp;
impl Operation for ConstOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Const is special: it reads from its own config, which is passed as "value" in inputs
        // (The engine merges config into inputs for simplicity in this architecture)
        let val: &Value = get_input(inputs, "value")?;
        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), val.clone());
        Ok(out)
    }
}

/// Print operation.
/// Inputs: `in`
/// Outputs: `out` (pass-through)
struct PrintOp;
impl Operation for PrintOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let val: &Value = get_input(inputs, "in")?;
        println!("OUTPUT: {}", val);
        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), val.clone());
        Ok(out)
    }
}

/// Mathematical operations (Add, Sub, Mul, Div).
/// Inputs: `a`, `b` (numbers)
/// Outputs: `out`
enum MathOp { Add, Sub, Mul, Div }
impl Operation for MathOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let a: f64 = get_f64(inputs, "a")?;
        let b: f64 = get_f64(inputs, "b")?;

        let res: f64 = match self {
            MathOp::Add => a + b,
            MathOp::Sub => a - b,
            MathOp::Mul => a * b,
            MathOp::Div => {
                if b == 0.0_f64 {
                    return Err(Error::OperationError {
                        node: "unknown".to_string(),
                        reason: "Division by zero".to_string(),
                    });
                }
                a / b
            }
        };

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), serde_json::json!(res));
        Ok(out)
    }
}

/// Logical comparison operations (Eq, Gt, Lt).
/// Inputs: `a`, `b`
/// Outputs: `out` (boolean)
enum LogicOp { Eq, Gt, Lt }
impl Operation for LogicOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let a: &Value = get_input(inputs, "a")?;
        let b: &Value = get_input(inputs, "b")?;

        let res: bool = match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                let f1: f64 = n1.as_f64().unwrap_or(0.0_f64); // Fallback to 0.0 if not a standard f64, though as_f64() should handle all numbers
                let f2: f64 = n2.as_f64().unwrap_or(0.0_f64);
                match self {
                    LogicOp::Eq => f1 == f2,
                    LogicOp::Gt => f1 > f2,
                    LogicOp::Lt => f1 < f2,
                }
            },
            _ => match self {
                LogicOp::Eq => a == b,
                _ => return Err(Error::InvalidType {
                    node: "unknown".to_string(),
                    expected: "comparable numbers".to_string(),
                    actual: "mixed/non-numeric types".to_string(),
                }),
            }
        };

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), serde_json::json!(res));
        Ok(out)
    }
}

/// Conditional switch operation.
/// Inputs: `cond` (bool), `data` (optional), `true` (optional), `false` (optional)
/// Outputs: `out`, `true` (conditional), `false` (conditional)
struct SwitchOp;
impl Operation for SwitchOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let cond: bool = get_bool(inputs, "cond")?;
        
        // Mode 1: Branch Selection (if true/false inputs exist)
        if inputs.contains_key("true") && inputs.contains_key("false") {
            let val: &Value = if cond {
                get_input(inputs, "true")?
            } else {
                get_input(inputs, "false")?
            };
            let mut out: HashMap<String, Value> = HashMap::new();
            let _: Option<Value> = out.insert("out".to_string(), val.clone());
            return Ok(out);
        }

        // Mode 2: Routing (requires 'data' input)
        let data: &Value = get_input(inputs, "data")?;
        let mut out: HashMap<String, Value> = HashMap::new();
        
        if cond {
            let _: Option<Value> = out.insert("true".to_string(), data.clone());
        } else {
            let _: Option<Value> = out.insert("false".to_string(), data.clone());
        }
        // Always pass through to 'out' for convenience
        let _: Option<Value> = out.insert("out".to_string(), data.clone());
        
        Ok(out)
    }
}

/// Array Map operation.
/// Applies an operation to every element in a list.
/// Inputs: 
/// - `list`: Array of values
/// - `op`: Name of operation to apply (e.g., "Add")
/// - `arg`: Name of the argument to inject the item into (e.g., "a")
/// - `params`: Optional static parameters for the operation (e.g., { "b": 1 })
/// Outputs: `out` (Array)
struct MapOp;
impl Operation for MapOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let list: &Vec<Value> = get_input(inputs, "list")?.as_array().ok_or_else(|| Error::InvalidType {
            node: "Map".to_string(),
            expected: "array".to_string(),
            actual: "non-array".to_string(),
        })?;
        
        let op_name: &str = get_input(inputs, "apply_op")?.as_str().ok_or_else(|| Error::InvalidType {
            node: "Map".to_string(),
            expected: "string (op name)".to_string(),
            actual: "non-string".to_string(),
        })?;
        
        let item_arg: &str = get_input(inputs, "arg")?.as_str().unwrap_or("in");
        
        // Static parameters to pass to every call
        let static_params: serde_json::Map<String, Value> = if let Some(params) = inputs.get("params") {
            params.as_object().ok_or_else(|| Error::InvalidType {
                node: "Map".to_string(),
                expected: "object (params)".to_string(),
                actual: "non-object".to_string(),
            })?.clone()
        } else {
            serde_json::Map::new()
        };

        let op: Box<dyn Operation> = Ops::get(op_name).ok_or_else(|| Error::UnknownOperation(op_name.to_string()))?;
        
        let mut result_list: Vec<Value> = Vec::new();
        
        for item in list {
            // Construct inputs for this iteration
            let mut op_inputs: HashMap<String, Value> = HashMap::new();
            // 1. Add static params
            for (k, v) in &static_params {
                let _: Option<Value> = op_inputs.insert(k.clone(), v.clone());
            }
            // 2. Add current item
            let _: Option<Value> = op_inputs.insert(item_arg.to_string(), item.clone());
            
            // Execute
            let op_result: HashMap<String, Value> = op.execute(&op_inputs)?;
            
            // Collect output (default to "out")
            let out_val: Value = op_result.get("out").unwrap_or(&Value::Null).clone();
            result_list.push(out_val);
        }

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), Value::Array(result_list));
        Ok(out)
    }
}

/// Array Reduce operation.
/// Reduces a list to a single value using an operation.
/// Inputs:
/// - `list`: Array of values
/// - `op`: Name of operation (e.g., "Add")
/// - `initial`: Initial accumulator value
/// - `acc_arg`: Argument name for accumulator (e.g., "a")
/// - `item_arg`: Argument name for item (e.g., "b")
/// Outputs: `out` (Value)
struct ReduceOp;
impl Operation for ReduceOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let list: &Vec<Value> = get_input(inputs, "list")?.as_array().ok_or_else(|| Error::InvalidType {
            node: "Reduce".to_string(),
            expected: "array".to_string(),
            actual: "non-array".to_string(),
        })?;
        
        let op_name: &str = get_input(inputs, "apply_op")?.as_str().ok_or_else(|| Error::InvalidType {
            node: "Reduce".to_string(),
            expected: "string (op name)".to_string(),
            actual: "non-string".to_string(),
        })?;
        
        let mut acc: Value = get_input(inputs, "initial")?.clone();
        let acc_arg: &str = get_input(inputs, "acc_arg")?.as_str().unwrap_or("a");
        let item_arg: &str = get_input(inputs, "item_arg")?.as_str().unwrap_or("b");

        let op: Box<dyn Operation> = Ops::get(op_name).ok_or_else(|| Error::UnknownOperation(op_name.to_string()))?;

        for item in list {
            let mut op_inputs: HashMap<String, Value> = HashMap::new();
            let _: Option<Value> = op_inputs.insert(acc_arg.to_string(), acc.clone());
            let _: Option<Value> = op_inputs.insert(item_arg.to_string(), item.clone());
            
            let op_result: HashMap<String, Value> = op.execute(&op_inputs)?;
            acc = op_result.get("out").unwrap_or(&Value::Null).clone();
        }

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), acc);
        Ok(out)
    }
}

/// Array Length operation.
/// Returns the number of elements in a list.
/// Inputs: `list`
/// Outputs: `out` (number)
struct LenOp;
impl Operation for LenOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let list: &Vec<Value> = get_input(inputs, "list")?.as_array().ok_or_else(|| Error::InvalidType {
            node: "Len".to_string(),
            expected: "array".to_string(),
            actual: "non-array".to_string(),
        })?;

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), serde_json::json!(list.len()));
        Ok(out)
    }
}

/// Array Filter operation.
/// Keeps only elements that satisfy a condition.
/// Inputs:
/// - `list`: Array of values
/// - `apply_op`: Name of comparison operation (e.g., "Gt", "Eq")
/// - `arg`: Argument name for the item (e.g., "a")
/// - `params`: Static parameters for comparison (e.g., { "b": 10 })
/// Outputs: `out` (filtered array)
struct FilterOp;
impl Operation for FilterOp {
    fn execute(&self, inputs: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let list: &Vec<Value> = get_input(inputs, "list")?.as_array().ok_or_else(|| Error::InvalidType {
            node: "Filter".to_string(),
            expected: "array".to_string(),
            actual: "non-array".to_string(),
        })?;
        
        let op_name: &str = get_input(inputs, "apply_op")?.as_str().ok_or_else(|| Error::InvalidType {
            node: "Filter".to_string(),
            expected: "string (op name)".to_string(),
            actual: "non-string".to_string(),
        })?;
        
        let item_arg: &str = get_input(inputs, "arg")?.as_str().unwrap_or("a");
        
        // Static parameters for the comparison
        let static_params: serde_json::Map<String, Value> = if let Some(params) = inputs.get("params") {
            params.as_object().ok_or_else(|| Error::InvalidType {
                node: "Filter".to_string(),
                expected: "object (params)".to_string(),
                actual: "non-object".to_string(),
            })?.clone()
        } else {
            serde_json::Map::new()
        };

        let op: Box<dyn Operation> = Ops::get(op_name).ok_or_else(|| Error::UnknownOperation(op_name.to_string()))?;
        
        let mut result_list: Vec<Value> = Vec::new();
        
        for item in list {
            // Construct inputs for this comparison
            let mut op_inputs: HashMap<String, Value> = HashMap::new();
            // 1. Add static params
            for (k, v) in &static_params {
                let _: Option<Value> = op_inputs.insert(k.clone(), v.clone());
            }
            // 2. Add current item
            let _: Option<Value> = op_inputs.insert(item_arg.to_string(), item.clone());
            
            // Execute comparison
            let op_result: HashMap<String, Value> = op.execute(&op_inputs)?;
            
            // Check if result is true
            let keep: bool = op_result.get("out")
                .and_then(|v: &Value| -> Option<bool> { v.as_bool() })
                .unwrap_or(false);
            
            if keep {
                result_list.push(item.clone());
            }
        }

        let mut out: HashMap<String, Value> = HashMap::new();
        let _: Option<Value> = out.insert("out".to_string(), Value::Array(result_list));
        Ok(out)
    }
}
