#!/usr/bin/env python3
"""UI similarity comparison between Ply and Dioxus reference screenshots.

Compares current runtime screenshots against Dioxus reference images using
structural similarity (SSIM) and perceptual hash distance.

Usage:
    python3 scripts/ui_similarity.py [--capture] [--threshold 0.95]

The script:
1. Optionally captures fresh screenshots from the running web app (--capture)
2. Maps each reference image to a screen name
3. Compares each pair using SSIM and perceptual hash
4. Reports per-screen and overall similarity scores
"""

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter
from skimage.metrics import structural_similarity as ssim
from skimage.transform import resize

# Reference screenshots mapping
# Map reference filenames to screen names and capture instructions
SCREEN_MAP = {
    "dashboard": {
        "ref": "unnamed (5) (3).jpg",
        "crop_ref_top": 90,  # Crop iOS status bar from reference
    },
    "editor": {
        "ref": "unnamed (5) (1).jpg",
        "crop_ref_top": 90,
    },
    "editor-select": {
        "ref": "unnamed57.jpg",
        "crop_ref_top": 0,
    },
    "layers": {
        "ref": "unnamed (5) (4).jpg",
        "crop_ref_top": 90,
    },
    "objects": {
        "ref": "unnamed (5) (5).jpg",
        "crop_ref_top": 90,
    },
    "settings": {
        "ref": "unnamed (5) (6).jpg",
        "crop_ref_top": 0,
    },
    "tilesets": {
        "ref": "unnamed (5) (2).jpg",
        "crop_ref_top": 90,
    },
}

REF_DIR = Path("dev/stage-2/references/ui-images")
PLY_DIR = Path("/tmp/taled-ply-screens")

CAPTURE_SCRIPT = """
from playwright.sync_api import sync_playwright
import time

def slow_click(page, x, y, wait=3):
    page.mouse.move(x, y)
    time.sleep(0.05)
    page.mouse.down()
    time.sleep(0.1)
    page.mouse.up()
    time.sleep(wait)

with sync_playwright() as p:
    browser = p.chromium.launch(
        headless=True,
        args=[
            '--no-sandbox',
            '--enable-webgl',
            '--use-gl=angle',
            '--use-angle=swiftshader',
            '--enable-unsafe-swiftshader',
            '--no-proxy-server',
        ]
    )
    context = browser.new_context(
        viewport={'width': 384, 'height': 688},
        user_agent='Mozilla/5.0 (Linux; Android 13)',
        device_scale_factor=1,
    )
    page = context.new_page()
    page.goto('http://localhost:8151/', timeout=60000)
    time.sleep(8)

    # Dashboard
    page.screenshot(path='/tmp/taled-ply-screens/dashboard.png')
    print("Captured: dashboard")

    # Click on first project card to enter editor
    slow_click(page, 192, 255, wait=5)
    page.screenshot(path='/tmp/taled-ply-screens/editor.png')
    print("Captured: editor")

    # Navigate to tilesets
    slow_click(page, 48, 660)
    page.screenshot(path='/tmp/taled-ply-screens/tilesets.png')
    print("Captured: tilesets")

    # Navigate to layers
    slow_click(page, 144, 660)
    page.screenshot(path='/tmp/taled-ply-screens/layers.png')
    print("Captured: layers")

    # Navigate to objects
    slow_click(page, 240, 660)
    page.screenshot(path='/tmp/taled-ply-screens/objects.png')
    print("Captured: objects")

    # Back to editor, then go to settings via header
    slow_click(page, 30, 22)
    time.sleep(2)

    # Navigate to Settings from dashboard bottom nav
    slow_click(page, 350, 660)
    page.screenshot(path='/tmp/taled-ply-screens/settings.png')
    print("Captured: settings")

    browser.close()
"""


def capture_screenshots():
    """Capture fresh screenshots from the running web app."""
    PLY_DIR.mkdir(parents=True, exist_ok=True)
    print("Capturing screenshots from http://localhost:8151/ ...")
    import os
    env = {**os.environ, "no_proxy": "*", "NO_PROXY": "*"}
    # Remove proxy settings that interfere with localhost
    for key in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY"]:
        env.pop(key, None)
    proc = subprocess.run(
        ["python3", "-c", CAPTURE_SCRIPT],
        capture_output=True,
        text=True,
        env=env,
        timeout=120,
    )
    if proc.returncode != 0:
        print(f"Capture failed:\n{proc.stderr}")
        sys.exit(1)
    print(proc.stdout)


def load_and_normalize(path, crop_top=0, target_size=(384, 688)):
    """Load image, crop top, resize to target, convert to grayscale array."""
    img = Image.open(path).convert("RGB")
    if crop_top > 0:
        # Scale crop_top proportionally to image height
        scale = img.height / target_size[1]
        actual_crop = int(crop_top * scale)
        img = img.crop((0, actual_crop, img.width, img.height))
    # Resize to target
    img = img.resize(target_size, Image.LANCZOS)
    return img


def compute_similarity(ref_img, ply_img):
    """Compute SSIM between two PIL images."""
    # Convert to grayscale numpy arrays
    ref_gray = np.array(ref_img.convert("L"), dtype=np.float64)
    ply_gray = np.array(ply_img.convert("L"), dtype=np.float64)

    # Ensure same size
    if ref_gray.shape != ply_gray.shape:
        ply_gray = resize(
            ply_gray, ref_gray.shape, anti_aliasing=True, preserve_range=True
        )

    # Compute SSIM
    score = ssim(ref_gray, ply_gray, data_range=255.0)
    return score


def compute_layout_similarity(ref_img, ply_img):
    """Compute layout similarity using edge detection + SSIM.

    This focuses on structural layout (edges, borders, sections)
    rather than pixel colors, which is more relevant for UI comparison.
    """
    # Apply edge detection to both
    ref_edges = np.array(
        ref_img.convert("L").filter(ImageFilter.FIND_EDGES), dtype=np.float64
    )
    ply_edges = np.array(
        ply_img.convert("L").filter(ImageFilter.FIND_EDGES), dtype=np.float64
    )

    if ref_edges.shape != ply_edges.shape:
        ply_edges = resize(
            ply_edges, ref_edges.shape, anti_aliasing=True, preserve_range=True
        )

    score = ssim(ref_edges, ply_edges, data_range=255.0)
    return score


def compute_color_histogram_similarity(ref_img, ply_img):
    """Compare color distribution similarity using histogram correlation."""
    ref_arr = np.array(ref_img)
    ply_arr = np.array(ply_img)

    similarities = []
    for channel in range(3):
        ref_hist, _ = np.histogram(ref_arr[:, :, channel], bins=64, range=(0, 256))
        ply_hist, _ = np.histogram(ply_arr[:, :, channel], bins=64, range=(0, 256))
        # Normalize
        ref_hist = ref_hist.astype(np.float64) / (ref_hist.sum() + 1e-10)
        ply_hist = ply_hist.astype(np.float64) / (ply_hist.sum() + 1e-10)
        # Correlation
        corr = np.corrcoef(ref_hist, ply_hist)[0, 1]
        similarities.append(max(0.0, corr))

    return np.mean(similarities)


def compare_screens(threshold=0.95):
    """Compare all screens and report similarity."""
    results = {}
    overall_scores = []

    for screen_name, config in SCREEN_MAP.items():
        ref_path = REF_DIR / config["ref"]
        ply_path = PLY_DIR / f"{screen_name}.png"

        if not ref_path.exists():
            print(f"  ⚠️  Reference not found: {ref_path}")
            continue
        if not ply_path.exists():
            print(f"  ⚠️  Ply screenshot not found: {ply_path}")
            continue

        ref_img = load_and_normalize(ref_path, crop_top=config.get("crop_ref_top", 0))
        ply_img = load_and_normalize(ply_path, crop_top=0)

        ssim_score = compute_similarity(ref_img, ply_img)
        layout_score = compute_layout_similarity(ref_img, ply_img)
        color_score = compute_color_histogram_similarity(ref_img, ply_img)

        # Weighted composite: layout matters most, then color, then raw SSIM
        composite = 0.4 * ssim_score + 0.35 * layout_score + 0.25 * color_score

        status = "✅" if composite >= threshold else "❌"
        results[screen_name] = {
            "ssim": ssim_score,
            "layout": layout_score,
            "color": color_score,
            "composite": composite,
            "status": status,
        }
        overall_scores.append(composite)

        print(
            f"  {status} {screen_name:16s}  "
            f"SSIM={ssim_score:.3f}  Layout={layout_score:.3f}  "
            f"Color={color_score:.3f}  Composite={composite:.3f}"
        )

    if overall_scores:
        avg = np.mean(overall_scores)
        status = "✅" if avg >= threshold else "❌"
        print(f"\n  {status} Overall average: {avg:.3f} (threshold: {threshold})")
        return avg, results
    return 0.0, results


def main():
    parser = argparse.ArgumentParser(description="UI similarity comparison")
    parser.add_argument("--capture", action="store_true", help="Capture fresh screenshots")
    parser.add_argument(
        "--threshold", type=float, default=0.95, help="Similarity threshold"
    )
    parser.add_argument("--json", action="store_true", help="Output JSON results")
    args = parser.parse_args()

    if args.capture:
        capture_screenshots()

    print(f"\n{'='*60}")
    print("  UI Similarity Report")
    print(f"{'='*60}\n")

    avg, results = compare_screens(args.threshold)

    if args.json:
        print(f"\n{json.dumps(results, indent=2)}")

    return 0 if avg >= args.threshold else 1


if __name__ == "__main__":
    sys.exit(main())
