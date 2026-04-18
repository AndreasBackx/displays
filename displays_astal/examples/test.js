import Gio from "gi://Gio"
import GLib from "gi://GLib"
import GIRepository from "gi://GIRepository"

// This smoke test is intended for the fake backend build. Override the build
// directory with DISPLAYS_ASTAL_BUILD_DIR if needed.
const candidateBuildDirs = [
  GLib.getenv("DISPLAYS_ASTAL_BUILD_DIR"),
  `${GLib.get_current_dir()}/build-faked`,
].filter(dir => dir && GLib.file_test(dir, GLib.FileTest.IS_DIR))

const buildDir = candidateBuildDirs[0]
if (!buildDir)
  throw new Error("could not find a fake DisplaysAstal build directory")

const repository = GIRepository.Repository.dup_default()
repository.prepend_search_path(buildDir)
repository.prepend_library_path(buildDir)

const Displays = imports.gi.DisplaysAstal

Gio._promisify(Displays.Manager.prototype, "query_async", "query_finish")
Gio._promisify(Displays.Manager.prototype, "get_async", "get_finish")
Gio._promisify(Displays.Manager.prototype, "validate_async", "validate_finish")

const manager = new Displays.Manager()

const displays = await manager.query_async(null)
if (displays.length !== 3)
  throw new Error(`expected 3 fake displays, got ${displays.length}`)

const first = displays[0]
if (first.id.name !== "Dell U2720Q")
  throw new Error(`unexpected first display name: ${first.id.name}`)

if (!first.logical.has_width || first.logical.width !== 3840)
  throw new Error("expected width on first logical display")

const matches = await manager.get_async([
  new Displays.DisplayIdentifier({ name: "LG UltraFine" }),
], null)

if (matches.length !== 1)
  throw new Error(`expected 1 match, got ${matches.length}`)

const unresolved = await manager.validate_async([
  new Displays.DisplayUpdate({
    id: new Displays.DisplayIdentifier({ name: "Missing Display" }),
    physical: new Displays.PhysicalDisplayUpdateContent({
      has_brightness: true,
      brightness: 50,
    }),
  }),
], null)

if (unresolved.length !== 1)
  throw new Error(`expected unresolved fake update, got ${unresolved.length}`)

print("DisplaysAstal fake backend smoke test passed")
