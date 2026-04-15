import GLib from "gi://GLib"
import GIRepository from "gi://GIRepository"

// This smoke test is intended for the fake backend build. Override the build
// directory with ASTAL_DISPLAYS_BUILD_DIR if needed.
const buildDir = GLib.getenv("ASTAL_DISPLAYS_BUILD_DIR")
    ?? "/home/andreas/dev/displays/displays-astal/build-faked"

GIRepository.Repository.dup_default().prepend_library_path(
    buildDir,
)

const Displays = imports.gi.AstalDisplays

const manager = new Displays.Manager()

const displays = manager.query()
if (displays.length !== 3)
    throw new Error(`expected 3 fake displays, got ${displays.length}`)

const first = displays[0]
if (first.id.name !== "Dell U2720Q")
    throw new Error(`unexpected first display name: ${first.id.name}`)

if (!first.logical.has_width || first.logical.width !== 3840)
    throw new Error("expected width on first logical display")

const matches = manager.get([
    new Displays.DisplayIdentifier({ name: "LG UltraFine" }),
])

if (matches.length !== 1)
    throw new Error(`expected 1 match, got ${matches.length}`)

const unresolved = manager.validate([
    new Displays.DisplayUpdate({
        id: new Displays.DisplayIdentifier({ name: "Missing Display" }),
        physical: new Displays.PhysicalDisplayUpdateContent({
            has_brightness: true,
            brightness: 50,
        }),
    }),
])

if (unresolved.length !== 1)
    throw new Error(`expected unresolved fake update, got ${unresolved.length}`)

print("AstalDisplays fake backend smoke test passed")
