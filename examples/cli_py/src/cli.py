from __future__ import annotations

import sys
from collections.abc import Sequence

import click
import displays


def _is_linux() -> bool:
    return sys.platform.startswith("linux")


def _format_resolution(display: displays.Display) -> str:
    width = display.logical.width
    height = display.logical.height
    left = str(width) if width is not None else "unknown"
    right = str(height) if height is not None else "unknown"
    return f"{left}x{right}"


def _format_brightness(display: displays.Display) -> str:
    if display.physical is None:
        return "unavailable"
    return f"{display.physical.brightness}%"


def _display_name(identifier: displays.DisplayIdentifier) -> str:
    return identifier.name or (identifier.serial_number or "unknown display")


def _target_name(identifier: displays.DisplayIdentifier) -> str:
    serial = identifier.serial_number
    if serial == "":
        serial = None
    return identifier.name or serial or "unknown display"


def _logical_update(
    is_enabled: bool | None,
    width: int | None,
    height: int | None,
) -> displays.LogicalDisplayUpdateContent | None:
    if is_enabled is None and width is None and height is None:
        return None

    return displays.LogicalDisplayUpdateContent(
        is_enabled=is_enabled,
        width=width,
        height=height,
    )


def _physical_update(
    brightness: int | None,
) -> displays.PhysicalDisplayUpdateContent | None:
    if brightness is None:
        return None

    return displays.PhysicalDisplayUpdateContent(brightness=brightness)


def _parse_position(
    _ctx: click.Context, _param: click.Parameter, value: str | None
) -> displays.Point | None:
    if value is None:
        return None

    left, sep, right = value.partition(",")
    if not sep:
        raise click.BadParameter("expected POSITION in the form x,y")

    try:
        return displays.Point(x=int(left), y=int(right))
    except ValueError as exc:
        raise click.BadParameter("expected POSITION in the form x,y") from exc


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.option("--enabled", "is_enabled", type=bool)
def query(is_enabled: bool | None) -> None:
    if _is_linux() and is_enabled is not None:
        raise click.ClickException(
            "--enabled uses logical display state and is not supported on Linux"
        )

    found_match = False
    for display in displays.query():
        if is_enabled is not None and display.logical.is_enabled != is_enabled:
            continue

        found_match = True
        identifier = display.id
        serial_number = identifier.serial_number or "unknown"

        click.echo(
            "\n".join(
                [
                    f"Display: {identifier.name or 'unknown'}",
                    f"  Serial: {serial_number}",
                    f"  Enabled: {display.logical.is_enabled}",
                    f"  Resolution: {_format_resolution(display)}",
                    f"  Brightness: {_format_brightness(display)}",
                    "",
                ]
            )
        )

    if not found_match:
        click.echo("No displays matched the query.")


@cli.command()
@click.option("--name")
@click.option("--serial-number")
@click.option("--is-enabled", type=bool)
@click.option("--width", type=int)
@click.option("--height", type=int)
@click.option("--position", callback=_parse_position)
@click.option("--brightness", type=int)
@click.option("--validate", is_flag=True)
def apply(
    name: str | None,
    serial_number: str | None,
    is_enabled: bool | None,
    width: int | None,
    height: int | None,
    position: displays.Point | None,
    brightness: int | None,
    validate: bool,
) -> None:
    logical = _logical_update(is_enabled=is_enabled, width=width, height=height)
    physical = _physical_update(brightness=brightness)

    if _is_linux() and logical is not None:
        raise click.ClickException("logical display updates are not supported on Linux")

    # Keep parity with the Rust CLI example, which parses --position but does not apply it yet.
    _ = position

    update = displays.DisplayUpdate(
        displays.DisplayIdentifier(name=name, serial_number=serial_number),
        logical=logical,
        physical=physical,
    )
    results: Sequence[displays.DisplayUpdateResult]
    results = displays.validate([update]) if validate else displays.apply([update])

    for result in results:
        target = _target_name(result.requested_update.id)

        if not result.applied and not result.failed:
            click.echo(f"No displays matched {target}.")
            continue

        if not result.failed:
            verb = "validated" if validate else "updated"
            click.echo(
                f"Successfully {verb} {len(result.applied)} display(s) for {target}."
            )
            continue

        if result.applied:
            click.echo(
                f"Partially applied {target}: {len(result.applied)} succeeded, {len(result.failed)} failed."
            )
        else:
            click.echo(f"Failed to apply {target}:")

        for failure in result.failed:
            click.echo(f"- {_display_name(failure.matched_id)}: {failure.message}")


def main() -> None:
    cli()


if __name__ == "__main__":
    main()
