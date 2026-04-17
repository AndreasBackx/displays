# displays_types

`displays_types` is the shared base layer for common display types across the workspace. Domain-specific type crates build on top of it: `displays_physical_types` should only be used by physical-display libraries, and the future logical equivalent should follow the same pattern.

```text
              +------------------------+
              |     displays_types     |
              +------------------------+
                 ^                 ^
                 |                 |
         +---------------+   +---------------------------+
         |   displays    |   |  displays_physical_types  |
         +---------------+   +---------------------------+
                                  ^                ^
                                  |                |
                    +------------------------+  +--------------------------+
                    | displays_physical_linux|  | displays_physical_windows|
                    +------------------------+  +--------------------------+

Future: a logical types crate should mirror displays_physical_types.
```
