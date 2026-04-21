## Published Crates

- `displays`: high-level cross-platform API for querying and updating displays
- `displays_types`: shared base types such as `DisplayIdentifier`, `Point`, and `Size`
- `displays_logical_types`: shared logical-display domain types
- `displays_physical_types`: shared physical-display domain types
- `displays_logical_linux`: Linux logical display support via Wayland and wlr output management
- `displays_physical_linux_sys`: low-level Linux sysfs brightness backend
- `displays_physical_linux_logind`: Linux brightness updates through systemd-logind
- `displays_physical_linux`: Linux physical brightness backend orchestration
- `displays_windows_common`: shared Windows display helpers
- `displays_logical_windows`: Windows logical display querying and updates
- `displays_physical_windows`: Windows physical brightness support
- `displays_py`: PyO3-based Python bindings exposing the `displays` module
- `displays_astal`: GLib/GObject bindings around `displays`
