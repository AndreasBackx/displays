# displays-astal

Astal/GI library for display control, implemented in Rust.

`displays-astal` exports a GObject-Introspection typelib that can be consumed
from GJS and TypeScript while delegating the actual display work to the Rust
`displays` crate.

## Backend Selection

The library supports two backend modes:

- default: real display queries and updates through `displays::DisplayManager`
- `faked` feature: deterministic fake data for smoke testing and typelib work

The fake backend is intentionally kept in-tree so the GI surface can be tested
without touching real hardware.

## Building

Default build, using the real backend:

```sh
cargo build -p astal_displays --release
meson setup build
meson compile -C build
```

Fake build, using the `faked` Cargo feature:

```sh
cargo build -p astal_displays --release --features faked
meson setup build-faked -Dfaked=true
meson compile -C build-faked
```

## Smoke Testing

The repository includes a fake-backend GJS smoke test at
`examples/test.js`. It expects a typelib built with the `faked` feature.

```sh
env GI_TYPELIB_PATH="$PWD/build-faked" gjs -m "$PWD/examples/test.js"
```

## API Notes

Some nullable scalar fields are exposed as paired `has_*` plus value properties,
for example `has_width` + `width`. This keeps the GI surface predictable for
TypeScript and avoids sentinel values or `GLib.Variant` wrappers for scalar
optionals.
