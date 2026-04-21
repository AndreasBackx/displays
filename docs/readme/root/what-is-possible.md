`displays` lets you query and update logical and physical display information on
Linux and Windows. Logical display state covers things like enabled state,
orientation, resolution, and placement. Physical display state currently focuses
on brightness.

The workspace is split into a small set of focused crates: shared type crates,
Linux and Windows backends, the top-level `displays` crate, and bindings for
Python and GLib/GObject consumers.

The most useful workflows today are:

- querying the current logical and physical display state
- matching displays by a friendly identifier such as name or serial number
- validating updates before applying them
- applying brightness updates on Linux and Windows
- consuming the same high-level API from Rust, Python, or GI-based environments

There is also a small CLI example in `examples/cli` for local experimentation.
