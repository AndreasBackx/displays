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
