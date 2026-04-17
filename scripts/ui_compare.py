#!/usr/bin/env python3

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter, ImageOps


def usage() -> None:
    print(
        "usage: ui_compare.py SCREEN RUNTIME REFERENCE DIFF COMPOSITE REPORT [THRESHOLD]",
        file=sys.stderr,
    )


def load_normalized(path: Path, ref_top: int) -> Image.Image:
    image = Image.open(path).convert("RGB")
    if ref_top:
        image = image.crop((0, ref_top, image.width, image.height))
    return image


def main() -> int:
    if len(sys.argv) not in {7, 8}:
        usage()
        return 2

    screen = sys.argv[1]
    runtime_path = Path(sys.argv[2])
    reference_path = Path(sys.argv[3])
    diff_path = Path(sys.argv[4])
    composite_path = Path(sys.argv[5])
    report_path = Path(sys.argv[6])
    threshold = float(sys.argv[7]) if len(sys.argv) == 8 else 0.12

    reference = load_normalized(reference_path, 44)
    runtime = load_normalized(runtime_path, 0)
    runtime = ImageOps.fit(runtime, reference.size, Image.Resampling.LANCZOS)

    blurred_runtime = runtime.filter(ImageFilter.GaussianBlur(radius=1.2))
    blurred_reference = reference.filter(ImageFilter.GaussianBlur(radius=1.2))
    diff = ImageChops.difference(blurred_runtime, blurred_reference)

    histogram = diff.histogram()
    squared_error = 0.0
    total_channels = reference.width * reference.height * 3
    for value, count in enumerate(histogram):
        channel_value = value % 256
        squared_error += (channel_value ** 2) * count

    rmse = math.sqrt(squared_error / total_channels) / 255.0
    diff.save(diff_path)

    composite = Image.new("RGB", (reference.width * 3, reference.height))
    composite.paste(reference, (0, 0))
    composite.paste(runtime, (reference.width, 0))
    composite.paste(diff, (reference.width * 2, 0))
    composite.save(composite_path)

    report = {
        "screen": screen,
        "runtime": str(runtime_path),
        "reference": str(reference_path),
        "diff": str(diff_path),
        "composite": str(composite_path),
        "rmse": round(rmse, 6),
        "threshold": threshold,
        "passed": rmse <= threshold,
    }
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
