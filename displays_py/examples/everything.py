import asyncio
import sys

from displays import *


async def main() -> None:
    print(query())

    ids = [display.id for display in query()]
    up_brightness_updates = [
        DisplayUpdate(id=id, physical=PhysicalDisplayUpdateContent(brightness=50))
        for id in ids
    ]
    print(apply(up_brightness_updates))

    down_brightness_updates = [
        DisplayUpdate(id=id, physical=PhysicalDisplayUpdateContent(brightness=0))
        for id in ids
    ]
    print(apply(down_brightness_updates))

    await asyncio.sleep(1)


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
