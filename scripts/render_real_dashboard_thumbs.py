#!/usr/bin/env python3

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import xml.etree.ElementTree as ET

from PIL import Image, ImageOps


ROOT = Path(__file__).resolve().parents[1]


@dataclass
class Tileset:
    firstgid: int
    tilewidth: int
    tileheight: int
    columns: int
    image: Image.Image


def parse_csv(data_text: str) -> list[int]:
    values: list[int] = []
    for raw in data_text.replace("\n", "").split(","):
        raw = raw.strip()
        if raw:
            values.append(int(raw))
    return values


def load_tileset(tsx_path: Path, firstgid: int) -> Tileset:
    root = ET.parse(tsx_path).getroot()
    tilewidth = int(root.get("tilewidth", "0"))
    tileheight = int(root.get("tileheight", "0"))
    image_node = root.find("./image")
    if image_node is None:
        raise ValueError(f"tileset image missing: {tsx_path}")
    image_path = (tsx_path.parent / image_node.get("source", "")).resolve()
    image = Image.open(image_path).convert("RGBA")
    columns_attr = root.get("columns")
    columns = int(columns_attr) if columns_attr else image.width // tilewidth
    return Tileset(firstgid, tilewidth, tileheight, columns, image)


def select_tileset(gid: int, tilesets: list[Tileset]) -> Tileset | None:
    selected: Tileset | None = None
    for tileset in tilesets:
        if gid >= tileset.firstgid:
            selected = tileset
        else:
            break
    return selected


def render_map(map_path: Path) -> Image.Image:
    root = ET.parse(map_path).getroot()
    width = int(root.get("width", "0"))
    height = int(root.get("height", "0"))
    tilewidth = int(root.get("tilewidth", "0"))
    tileheight = int(root.get("tileheight", "0"))

    tilesets: list[Tileset] = []
    for node in root.findall("./tileset"):
        firstgid = int(node.get("firstgid", "0"))
        source = node.get("source")
        if source is None:
            continue
        tilesets.append(load_tileset((map_path.parent / source).resolve(), firstgid))

    tilesets.sort(key=lambda item: item.firstgid)
    canvas = Image.new("RGBA", (width * tilewidth, height * tileheight), (0, 0, 0, 0))

    for layer in root.findall("./layer"):
        data_node = layer.find("./data")
        if data_node is None or data_node.get("encoding") != "csv":
            continue
        gids = parse_csv(data_node.text or "")
        for index, gid in enumerate(gids):
            if gid == 0:
                continue
            tileset = select_tileset(gid, tilesets)
            if tileset is None:
                continue
            local_id = gid - tileset.firstgid
            sx = (local_id % tileset.columns) * tileset.tilewidth
            sy = (local_id // tileset.columns) * tileset.tileheight
            tile = tileset.image.crop((sx, sy, sx + tileset.tilewidth, sy + tileset.tileheight))
            dx = (index % width) * tilewidth
            dy = (index // width) * tileheight
            canvas.alpha_composite(tile, (dx, dy))

    bbox = canvas.getbbox()
    if bbox:
        left, top, right, bottom = bbox
        pad = max(tilewidth, tileheight) // 2
        canvas = canvas.crop(
            (
                max(0, left - pad),
                max(0, top - pad),
                min(canvas.width, right + pad),
                min(canvas.height, bottom + pad),
            )
        )

    return canvas


def save_thumb(source: Path, output: Path) -> None:
    rendered = render_map(source)
    thumb = ImageOps.fit(rendered.convert("RGB"), (96, 96), Image.Resampling.LANCZOS)
    output.parent.mkdir(parents=True, exist_ok=True)
    thumb.save(output)


def main() -> None:
    jobs = [
        (
            ROOT / "assets/samples/stage1-basic/map.tmx",
            ROOT / "assets/review/dashboard-stage1.png",
        ),
        (
            ROOT / "assets/samples/tmwa/maps/017-2.tmx",
            ROOT / "assets/review/dashboard-theater.png",
        ),
        (
            ROOT / "assets/samples/tmwa/maps/081-3.tmx",
            ROOT / "assets/review/dashboard-frontier.png",
        ),
    ]

    for source, output in jobs:
        save_thumb(source, output)
        print(f"wrote {output.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
