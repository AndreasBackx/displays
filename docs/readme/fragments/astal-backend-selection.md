## Backend Selection

The library supports two backend modes:

- default: real display queries and updates through `displays::DisplayManager`
- `faked` feature: deterministic fake data for smoke testing and typelib work

The fake backend is intentionally kept in-tree so the GI surface can be tested
without touching real hardware.
