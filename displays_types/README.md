<!-- This file is generated from crate docs with cargo-readme. Do not edit directly. -->

# displays_types

`displays_types` is the shared base layer for common display types across the workspace.

Domain-specific type crates build on top of it: `displays_logical_types` for logical display state and `displays_physical_types` for physical display state.

Most users should start with the top-level `displays` crate instead of depending on this crate directly.

```
                 +------------------------+
                 |     displays_types     |
                 +------------------------+
                    ^        ^        ^
                    |        |        |
          +---------------+  |  +---------------------------+
          |   displays    |  |  |  displays_logical_types   |
          +---------------+  |  +---------------------------+
                 +---------------------------+
                 |  displays_phyiscal_types  |
                 +---------------------------+
                       ^                ^
                       |                |
        +-------------------------+  +---------------------------+
        | displays_physical_linux |  | displays_physical_windows |
        +-------------------------+  +---------------------------+
```
