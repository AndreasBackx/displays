<!-- This file is generated from crate docs with cargo-readme. Do not edit directly. -->

# displays_astal

Astal/GI library for display control, implemented in Rust.

`displays_astal` exports a GObject-Introspection typelib that can be consumed
from GJS and TypeScript while delegating the actual display work to the Rust
`displays` crate.

See the project root `README.md` for the overall crate graph and the main
cross-platform Rust API.

## Backend Selection

The library supports two backend modes:

- default: real display queries and updates through `displays::DisplayManager`
- `faked` feature: deterministic fake data for smoke testing and typelib work

The fake backend is intentionally kept in-tree so the GI surface can be tested
without touching real hardware.

## Building

Default build, using the real backend:

```sh
cargo build -p displays_astal --release
meson setup build
meson compile -C build
```

Fake build, using the `faked` Cargo feature:

```sh
cargo build -p displays_astal --release --features faked
meson setup build-faked -Dfaked=true
meson compile -C build-faked
```

## Smoke Testing

The repository includes a fake-backend GJS smoke test at `examples/test.js`.
It expects a typelib built with the `faked` feature.

```sh
env GI_TYPELIB_PATH="$PWD/build-faked" gjs -m "$PWD/examples/test.js"
```

## Astal / GI Example

`displays_astal` exposes an async GI API that can be consumed from GJS or
TypeScript while delegating the actual display work to the Rust `displays`
crate.

```ts
import Gio from "gi://Gio";
import GLib from "gi://GLib";
import DisplaysAstal from "gi://DisplaysAstal";

Gio._promisify(DisplaysAstal.Manager.prototype, "query_async", "query_finish");
Gio._promisify(DisplaysAstal.Manager.prototype, "get_async", "get_finish");
Gio._promisify(DisplaysAstal.Manager.prototype, "update_async", "update_finish");
Gio._promisify(DisplaysAstal.Manager.prototype, "validate_async", "validate_finish");

async function main() {
    const manager = DisplaysAstal.Manager.get_default();

    const displays = await manager.query_async(null);
    for (const display of displays) {
        const name = display.id.name ?? "unknown";
        const serial = display.id.serial_number ?? "unknown";
        print(`${name} (${serial})`);
    }

    const results = await manager.update_async([
        new DisplaysAstal.DisplayUpdate({
            id: new DisplaysAstal.DisplayIdentifier({ name: "Missing Display" }),
            physical: new DisplaysAstal.PhysicalDisplayUpdateContent({
                has_brightness: true,
                brightness: 50,
            }),
        }),
    ], null);

    print(`apply returned ${results.length} result(s)`);
}

main().catch(err => {
    printerr(`DisplaysAstal error: ${err.message}`);
    GLib.exit(1);
});
```

## API Notes

Some nullable scalar fields are exposed as paired `has_*` plus value properties,
for example `has_width` + `width`. This keeps the GI surface predictable for
TypeScript and avoids sentinel values or `GLib.Variant` wrappers for scalar
optionals.
