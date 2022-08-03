This crate generates Rust structures from OpenAPI 3.0 definitions.

## Example

```rust
use openapi_struct_gen::generate;

fn main() {
    generate(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/example-schema.yaml"),
        concat!(env!("CARGO_OUT_DIR"), "/gen.rs"),
        &["Clone", "Serialize", "Deserialize"],
        &[("serde", "Serialize"), ("serde", "Deserialize")],
    )
    .unwrap();
}
```

## Goals
* Generate Rust structures from Open API 3.0 definitions

## Non Goals
* Generate web servers and clients

