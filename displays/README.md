<!-- This file is generated from crate docs with cargo-readme. Do not edit directly. -->

# displays

`displays` is the high-level crate for querying and updating logical and physical display state on Linux and Windows.

It builds on the lower-level `displays_*` crates in this workspace and is the main entry point most users should start with.

This crate is the main Rust API for the workspace and the entry point most users should use.

## Rust Example

The top-level `displays` crate can query display state, match displays by a
user-facing identifier, and apply updates.

```rust
use displays::{
    display::DisplayUpdate,
    manager::DisplayManager,
    types::{DisplayIdentifier, PhysicalDisplayUpdateContent},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let displays = DisplayManager::query()?;

    for display in &displays {
        println!("{display:#?}");
    }

    let results = DisplayManager::apply(vec![DisplayUpdate {
        id: DisplayIdentifier {
            name: Some("DELL U2720Q".to_string()),
            serial_number: None,
        },
        logical: None,
        physical: Some(PhysicalDisplayUpdateContent {
            brightness: Some(50),
        }),
    }], false)?;

    assert_eq!(results.len(), 1);
    Ok(())
}
```
