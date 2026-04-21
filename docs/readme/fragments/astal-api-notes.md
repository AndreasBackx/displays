## API Notes

Some nullable scalar fields are exposed as paired `has_*` plus value properties,
for example `has_width` + `width`. This keeps the GI surface predictable for
TypeScript and avoids sentinel values or `GLib.Variant` wrappers for scalar
optionals.
