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
