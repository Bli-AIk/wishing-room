#!/usr/bin/env python3

from __future__ import annotations

import shutil
from pathlib import Path
import xml.etree.ElementTree as ET


def copy_tree(src: Path, dst: Path) -> None:
    for path in src.rglob("*"):
        relative = path.relative_to(src)
        target = dst / relative
        if path.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    source_res = (
        root / "apps" / "taled-editor" / "android" / "app" / "src" / "main" / "res"
    )
    target_main = (
        root
        / "target"
        / "dx"
        / "taled-editor"
        / "release"
        / "android"
        / "app"
        / "app"
        / "src"
        / "main"
    )

    if not target_main.exists():
        raise SystemExit(
            "Android build output is missing. Run `dx build --android --target aarch64-linux-android -r -p taled-editor` first."
        )

    copy_tree(source_res, target_main / "res")

    manifest_path = target_main / "AndroidManifest.xml"
    android_ns = "http://schemas.android.com/apk/res/android"
    ET.register_namespace("android", android_ns)
    tree = ET.parse(manifest_path)
    root_el = tree.getroot()
    application = root_el.find("application")
    if application is None:
        raise SystemExit(f"Android manifest is missing <application>: {manifest_path}")
    round_icon_key = f"{{{android_ns}}}roundIcon"
    if application.get(round_icon_key) != "@mipmap/ic_launcher_round":
        application.set(round_icon_key, "@mipmap/ic_launcher_round")
        tree.write(manifest_path, encoding="utf-8", xml_declaration=True)


if __name__ == "__main__":
    main()
