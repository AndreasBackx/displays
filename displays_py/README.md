<!-- This file is generated from crate docs with cargo-readme. Do not edit directly. -->

# displays_py

`displays_py` is the Rust crate backing the Python extension module named `displays`.

For the main Rust API, start with the `displays` crate.

## Python Setup

For local development inside `displays_py/`:

```bash
uv sync --reinstall-package displays
uv run ipython
```

## Python Example

The `displays_py` crate exposes the Python module name `displays` with the same
high-level `query`, `get`, `apply`, and `validate` operations.

```python
import displays

for display in displays.query():
    print(display)

results = displays.apply([
    displays.DisplayUpdate(
        id=displays.DisplayIdentifier(name="DELL U2720Q"),
        physical=displays.PhysicalDisplayUpdateContent(brightness=50),
    ),
])

print(results)
```
