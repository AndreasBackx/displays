Astal/GI library for display control, implemented in Rust.

`displays_astal` exports a GObject-Introspection typelib that can be consumed
from GJS and TypeScript while delegating the actual display work to the Rust
`displays` crate.

For the main cross-platform Rust API, start with the `displays` crate.
