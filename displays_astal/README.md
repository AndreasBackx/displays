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

## Async Usage

The GI API is async-only and follows the standard GLib `*_async` / `*_finish`
pattern. In GJS and TypeScript, the usual approach is to promisify the methods
once and then `await` them.

```ts
import Gio from "gi://Gio";
import GLib from "gi://GLib";
import AstalDisplays from "gi://AstalDisplays";

Gio._promisify(AstalDisplays.Manager.prototype, "query_async", "query_finish");
Gio._promisify(AstalDisplays.Manager.prototype, "get_async", "get_finish");
Gio._promisify(AstalDisplays.Manager.prototype, "update_async", "update_finish");
Gio._promisify(AstalDisplays.Manager.prototype, "validate_async", "validate_finish");

async function main() {
    const manager = AstalDisplays.Manager.get_default();

    const displays = await manager.query_async(null);
    for (const display of displays) {
        const name = display.id.name ?? "unknown";
        const serial = display.id.serial_number ?? "unknown";
        print(`${name} (${serial})`);
    }

    const matches = await manager.get_async([
        new AstalDisplays.DisplayIdentifier({ name: "LG UltraFine" }),
    ], null);

    print(`matched ${matches.length} display(s)`);

    const validation = await manager.validate_async([
        new AstalDisplays.DisplayUpdate({
            id: new AstalDisplays.DisplayIdentifier({ name: "Missing Display" }),
            physical: new AstalDisplays.PhysicalDisplayUpdateContent({
                has_brightness: true,
                brightness: 50,
            }),
        }),
    ], null);

    print(`validation returned ${validation.length} result(s)`);
}

main().catch(err => {
    printerr(`AstalDisplays error: ${err.message}`);
    GLib.exit(1);
});
```

## API Notes

Some nullable scalar fields are exposed as paired `has_*` plus value properties,
for example `has_width` + `width`. This keeps the GI surface predictable for
TypeScript and avoids sentinel values or `GLib.Variant` wrappers for scalar
optionals.
