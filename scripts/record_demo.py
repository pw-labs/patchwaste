#!/usr/bin/env python3
"""Generate the patchwaste README demo video.

Renders scripts/demo_scenes.html in headless Chromium, captures the playback
as native browser video (continuous motion, not slideshow frames), then
transcodes to MP4 with ffmpeg.

Requires the project venv at .venv-demo/ with playwright + Pillow installed,
plus ffmpeg on PATH.

    python3 -m venv .venv-demo
    .venv-demo/bin/pip install playwright Pillow
    .venv-demo/bin/playwright install chromium
    .venv-demo/bin/python scripts/record_demo.py
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    from playwright.sync_api import sync_playwright
except ImportError:
    sys.exit("playwright not installed. See header for setup.")

ROOT = Path(__file__).resolve().parent.parent
SCENES_HTML = ROOT / "scripts" / "demo_scenes.html"
OUT_MP4 = ROOT / "demo.mp4"

WIDTH = 1280
HEIGHT = 720
DURATION_MS = 30_500


def capture_webm(target_dir: Path) -> Path:
    target_dir.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch()
        context = browser.new_context(
            viewport={"width": WIDTH, "height": HEIGHT},
            record_video_dir=str(target_dir),
            record_video_size={"width": WIDTH, "height": HEIGHT},
        )
        page = context.new_page()
        page.goto(SCENES_HTML.as_uri())
        page.wait_for_timeout(DURATION_MS)
        context.close()
        browser.close()
    webms = list(target_dir.glob("*.webm"))
    if not webms:
        sys.exit("playwright did not produce a video file")
    return webms[0]


def transcode(webm: Path, mp4: Path) -> None:
    if mp4.exists():
        mp4.unlink()
    subprocess.run(
        [
            "ffmpeg", "-y", "-loglevel", "error",
            "-i", str(webm),
            "-vf", f"scale={WIDTH}:{HEIGHT}:flags=lanczos",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "20",
            "-preset", "slow",
            "-movflags", "+faststart",
            "-an",
            str(mp4),
        ],
        check=True,
    )


def main() -> None:
    if not SCENES_HTML.exists():
        sys.exit(f"missing {SCENES_HTML}")
    if shutil.which("ffmpeg") is None:
        sys.exit("ffmpeg not on PATH")

    tmp = Path(tempfile.mkdtemp(prefix="pw_demo_"))
    try:
        print("recording chromium playback...")
        webm = capture_webm(tmp)
        print(f"  raw {webm.name} {webm.stat().st_size // 1024} kB")

        print("transcoding to mp4...")
        transcode(webm, OUT_MP4)
        print(f"wrote {OUT_MP4} ({OUT_MP4.stat().st_size // 1024} kB)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
