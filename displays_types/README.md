# displays_types

`displays_types` is the shared base layer for common display types across the workspace. Domain-specific type crates build on top of it: `displays_physical_types` should only be used by physical-display libraries, and `displays_logical_types` should only be used by logical-display libraries.

```text
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
