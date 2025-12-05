<p align="center">
  <img src="assets/spell.svg" alt="SPELL" width="100">
</p>

# SPELL

**The first programming language designed for AI.**

SPELL is an AI-native dataflow language where computation is expressed as a graph of explicit dependencies. No sequential reasoning. No implicit state. Just structure that mirrors logic.

> *Explicit dependencies, explicit types, structured format.*

📄 [Read the Paper](https://zenodo.org/records/17826541) 

## Status

```
⚠️  Pre-Alpha (v0.1)
```

This is a minimal proof-of-concept demonstrating the core dataflow paradigm. The current implementation validates the fundamental architecture.

**Not yet implemented:** Extended operation set, file I/O, network operations, string manipulation, custom function definitions, error recovery.

## Quick Start

```bash
# Run a program
cargo run -- examples/sales_analysis.json

# Run with debug output
cargo run -- examples/statistics.json --debug
```

## Philosophy

| Principle | Description |
|-----------|-------------|
| **Explicit Dependencies** | Each node declares its inputs. No hidden state. |
| **Explicit Types** | Every value has a type annotation. Types are stated, not inferred. |
| **Structured Format** | JSON. Rigid syntax. Native to LLM training data. |

## Example

```json
{
  "data": {
    "op": "Const",
    "value": { "literal": [1, 2, 3, 4, 5], "type": "Array<Number>" },
    "returns": "Array<Number>"
  },
  "sum": {
    "op": "Reduce",
    "list": { "ref": "data", "type": "Array<Number>" },
    "apply_op": { "literal": "Add", "type": "String" },
    "initial": { "literal": 0, "type": "Number" },
    "acc_arg": { "literal": "a", "type": "String" },
    "item_arg": { "literal": "b", "type": "String" },
    "returns": "Number"
  },
  "result": {
    "op": "Print",
    "in": { "ref": "sum", "type": "Number" },
    "returns": "Number"
  }
}
```

Each node has a name and declares:
- **op**: The operation to perform
- **inputs**: References (`ref`) or literals (`literal`), each with explicit type
- **returns**: The output type

## Types

| Type | Description |
|------|-------------|
| `Number` | Numeric values |
| `String` | Text values |
| `Boolean` | `true` or `false` |
| `Array<T>` | Ordered collection of type T |
| `Any` | Dynamic type |

## Operations (v0.1)

This minimal set is expressively complete for data transformations.

| Operation | Inputs | Output |
|-----------|--------|--------|
| `Const` | `value` | Value |
| `Add`, `Sub`, `Mul`, `Div` | `a`, `b` | Number |
| `Eq`, `Gt`, `Lt` | `a`, `b` | Boolean |
| `Map` | `list`, `apply_op`, `arg`, `params` | Array |
| `Filter` | `list`, `apply_op`, `arg`, `params` | Array |
| `Reduce` | `list`, `apply_op`, `initial`, `acc_arg`, `item_arg` | Value |
| `Len` | `list` | Number |
| `Switch` | `cond`, `true`, `false` | Value |
| `Print` | `in` | Value |

## Examples

See the [`examples/`](./examples) directory for complete programs:

- `sales_analysis.json` — Filter and aggregate sales data
- `statistics.json` — Calculate mean
- `temperature_conversion.json` — Batch data transformation

## Why SPELL?

Programming languages are abstractions designed for authors. Traditional languages are designed for human authors who think sequentially and track state mentally.

LLMs are different. They generate through pattern completion, not sequential reasoning. SPELL is an abstraction designed for this mechanism.

## Contact

📧 [research@santino.world](mailto:research@santino.world)

## License

MIT — [Santino Research]
