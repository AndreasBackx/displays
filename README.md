# displays

`displays` allows you to easily query and mutate logical and physical display information on Linux and Windows. Logical as the resolution of location of your displays. Physical as in the brightness of your monitor.

The project currently focuses on Linux and Windows support. There are Python bindings available as well, however they're not the main focus. They were originally made as I wanted to use it from Python, but that has changed.

Finally, there is also includes a small CLI example in `examples/cli` for local experimentation.

It's not yet recommended for day-to-day use, but I encourage you to try it out and experiment.

## Rust Usage

```rust
use displays::{
    display::DisplayUpdate,
    display_identifier::DisplayIdentifier,
    manager::DisplayManager,
    physical_display::PhysicalDisplayUpdateContent,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let displays = DisplayManager::query()?;

    for display in &displays {
        println!("{display:#?}");
    }

    let results = DisplayManager::apply(
        vec![DisplayUpdate {
            id: DisplayIdentifier {
                name: Some("DELL U2720Q".to_string()),
                serial_number: None,
            },
            logical: None,
            physical: Some(PhysicalDisplayUpdateContent {
                brightness: Some(50),
            }),
        }],
        false,
    )?;

    assert_eq!(results.len(), 1);
    assert!(!results[0].applied.is_empty());
    Ok(())
}
```

## Python Usage

The Python bindings live in `displays_py/` and expose the module name `displays`.

For local development:

```bash
cd displays_py
uv sync --reinstall-package displays
uv run python -c "import displays; print(displays.query())"
```

There is also a simple example script at `displays_py/examples/everything.py`.

## Support Matrix

| Platform | Logical | Logical | Logical | Logical | Physical |
| --- | --- | --- | --- | --- | --- |
|  | Enabled | Orientation | Resolution | Position | Brightness |
| Windows | Supported | Supported | Experimental | Experimental | Supported |
| Linux | Unsupported | Unsupported | Unsupported | Unsupported | Supported |
| macOS | Unsupported | Unsupported | Unsupported | Unsupported | Unsupported |
