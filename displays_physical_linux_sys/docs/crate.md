`displays_physical_linux_sys` is a small Rust library for querying and updating Linux
brightness devices exposed through sysfs.

It is also the lowest-level Linux physical brightness backend used by the higher-level `displays` crate.

Start with the `displays` crate unless you need direct sysfs access.

It currently works with devices under:

- `/sys/class/backlight`
- `/sys/class/leds`

The API is intentionally manager-oriented and library-focused. It exposes typed
device metadata and brightness update operations instead of mirroring a CLI.

## How this maps to Linux

On Linux, brightness for many devices is exposed through the kernel's sysfs
interface. Sysfs presents kernel objects as files and directories, usually under
`/sys`.

For this crate, the important directories are:

- `/sys/class/backlight` for laptop panels and other backlight-style devices
- `/sys/class/leds` for LED-style brightness devices, including some keyboard
  backlights and indicator LEDs

Each device normally appears as a directory such as:

- `/sys/class/backlight/intel_backlight`
- `/sys/class/leds/asus::kbd_backlight`

Inside that directory, Linux commonly exposes:

- `brightness`: the current raw brightness value that can be written
- `max_brightness`: the maximum allowed raw value
- `actual_brightness`: the effective brightness actually being applied, when the
  driver exposes it

This crate reads those files directly. That is why the API exposes raw values as
first-class data instead of hiding everything behind percentages.

## Understanding raw values

Linux brightness values are device-specific integers, not universal units.

For example:

- one device may use a range of `0..=255`
- another may use `0..=937`
- another may use `0..=3`

So `BrightnessUpdate::Raw(100)` does not mean the same thing across devices.
When you need a cross-device notion of brightness, use percentages.

The crate therefore exposes both:

- raw values, which match Linux directly
- `brightness_percent`, which is derived from `brightness / max_brightness`

## Reading versus actual brightness

Some drivers expose both `brightness` and `actual_brightness`.

- `brightness` is the configured target value
- `actual_brightness` is the effective value reported by the driver

They are often equal, but they do not have to be. For example, hardware or
driver behavior may temporarily clamp or lag behind the configured value.

If `actual_brightness` is not present, the device is still fully usable through
this crate.

## Permissions and writes

Reading sysfs brightness files is commonly allowed for regular users, but
writing is often restricted.

In practice, successful writes usually require one of these:

- root privileges
- udev rules that grant write access to the relevant `brightness` file
- a system policy that changes device permissions outside the crate

This first version performs direct sysfs writes only. It does not currently use
logind or another privileged mediation path.

## Why both backlight and leds are included

Linux does not have a single unified "brightness device" abstraction. Similar
controls can show up in different sysfs classes depending on the driver and the
hardware.

This crate includes both `backlight` and `leds` so callers can work with the
kernel's available brightness-capable devices through one API.

## Features

- Enumerate backlight and LED brightness devices
- Read current, maximum, and optional actual brightness values
- Match devices by class, id, and path
- Apply brightness updates using raw values, percentages, and deltas
- Validate updates without writing to sysfs
- Use a custom sysfs root for tests and fixtures

## Usage

```rust,no_run
use std::collections::BTreeSet;

use displays_physical_linux_sys::{
    BrightnessUpdate, DeviceClass, DeviceIdentifier, DeviceUpdate,
    PhysicalDisplayManagerLinuxSys,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PhysicalDisplayManagerLinuxSys::new();

    for device in manager.list()? {
        println!(
            "{} {}: {}/{} ({}%)",
            match device.metadata.class {
                DeviceClass::Backlight => "backlight",
                DeviceClass::Leds => "leds",
            },
            device.metadata.id,
            device.state.brightness_raw,
            device.state.max_brightness_raw,
            device.state.brightness_percent,
        );
    }

    let remaining = manager.update(vec![DeviceUpdate {
        id: DeviceIdentifier {
            class: Some(DeviceClass::Backlight),
            id: Some("intel_backlight".to_string()),
            path: None,
        },
        brightness: Some(BrightnessUpdate::Percent(50)),
    }])?;

    assert!(remaining.is_empty());
    Ok(())
}
```

## Matching

`DeviceIdentifier` uses subset matching:

- `class` restricts matching to one sysfs class
- `id` matches the device directory name
- `path` matches the full device path exactly

Any field left as `None` is ignored during matching.

## Update semantics

The crate currently supports these update types:

- `BrightnessUpdate::Raw(u32)`
- `BrightnessUpdate::Percent(u8)`
- `BrightnessUpdate::RawDelta(i32)`
- `BrightnessUpdate::PercentDelta(i32)`

All writes are clamped to the device's valid raw range `0..=max_brightness`.

## Linux behavior

The crate reads the same sysfs brightness values that Linux exposes directly.
That means raw values are first-class API data rather than being hidden behind a
percent-only abstraction.

If an `actual_brightness` file exists, it is exposed as
`DeviceState::actual_brightness_raw`. It is not required for a device to be
usable.

## Current scope

This first version uses direct sysfs reads and writes only.

It does not currently include:

- logind-based write fallback
- hotplug monitoring
- non-sysfs backends

## Further reading

Kernel documentation:

- Sysfs overview: <https://www.kernel.org/doc/html/latest/filesystems/sysfs.html>
- Backlight support: <https://www.kernel.org/doc/html/latest/gpu/backlight.html>
- LED class documentation: <https://www.kernel.org/doc/html/latest/leds/leds-class.html>

Man pages to check locally on a Linux system:

- `man 5 sysfs`
- `man 7 udev`
- `man 8 udevadm`

Those references are useful if you want to understand where these files come
from, why device permissions differ between systems, or how a particular driver
maps hardware brightness into sysfs.

## License

MIT
