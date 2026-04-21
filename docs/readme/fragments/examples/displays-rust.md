## Rust Example

The top-level `displays` crate can query display state, match displays by a
user-facing identifier, and apply updates.

```rust,no_run
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
