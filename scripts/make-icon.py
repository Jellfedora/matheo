#!/usr/bin/env python3
"""Draws Mathéo's icon: his terracotta pot with its face, five big leaves, on a soft green rounded square.

Writes assets/icon.png (1024 px), assets/Matheo.icns (macOS, needs iconutil) and assets/matheo.ico
(Windows). Requires Pillow.
"""
import math
import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets"
S = 1024
K = 4  # drawn 4 times larger, then scaled down: smooth edges


def draw() -> Image.Image:
    n = S * K
    img = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # The rounded square, light green to cream.
    m = 70 * K
    bg = Image.new("RGBA", (n, n))
    g = ImageDraw.Draw(bg)
    for y in range(n):
        t = y / n
        g.line([(0, y), (n, y)], fill=(int(214 + 34 * t), int(238 + 8 * t), int(200 + 30 * t), 255))
    mask = Image.new("L", (n, n), 0)
    ImageDraw.Draw(mask).rounded_rectangle([m, m, n - m, n - m], radius=200 * K, fill=255)
    img.paste(bg, (0, 0), mask)

    u = 8.0 * K  # Mathéo's unit: 70 units tall in the app
    foot = (n / 2, n - m - 130 * K)

    def at(x, y):
        return (foot[0] + x * u, foot[1] + y * u)

    # Leaves behind the pot, from the soil.
    leaves = [(-1.0, 30), (-0.45, 38), (0.1, 42), (0.6, 36), (1.1, 28)]
    colors = [(72, 160, 70), (90, 180, 80)]
    for k, (angle, length) in enumerate(leaves):
        a = -math.pi / 2 + angle * 0.8
        base = at((k - 2) * 3, -32)
        tip = (base[0] + math.cos(a) * length * u, base[1] + math.sin(a) * length * u)
        mid = (base[0] + (tip[0] - base[0]) * 0.55, base[1] + (tip[1] - base[1]) * 0.55)
        d.line([base, mid], fill=(60, 120, 50), width=int(2.4 * u))
        dx, dy = math.cos(a), math.sin(a)
        sx, sy = -dy, dx
        pts = []
        for i in range(25):
            f = i / 24
            w = math.sin(f * math.pi) * 9 * u
            pts.append((mid[0] + dx * f * length * 0.6 * u + sx * w, mid[1] + dy * f * length * 0.6 * u + sy * w))
        for i in range(24, -1, -1):
            f = i / 24
            w = math.sin(f * math.pi) * 9 * u
            pts.append((mid[0] + dx * f * length * 0.6 * u - sx * w, mid[1] + dy * f * length * 0.6 * u - sy * w))
        d.polygon(pts, fill=colors[k % 2], outline=(40, 100, 40), width=int(0.8 * u))
        vein_end = (mid[0] + dx * length * 0.55 * u, mid[1] + dy * length * 0.55 * u)
        d.line([mid, vein_end], fill=(140, 210, 120), width=int(0.7 * u))
    # Shadow, pot, rim, soil.
    d.ellipse([at(-20, -4), at(20, 4)], fill=(0, 0, 0, 50))
    d.polygon([at(-14, 0), at(14, 0), at(18, -26), at(-18, -26)], fill=(196, 104, 62))
    d.rounded_rectangle([at(-21, -32), at(21, -24)], radius=2 * u, fill=(214, 122, 76))
    d.rectangle([at(-19, -33), at(19, -31)], fill=(92, 62, 40))
    # His face.
    ink = (60, 36, 24)
    for s in (-1, 1):
        c = at(s * 6, -15)
        d.ellipse([c[0] - 2 * u, c[1] - 2 * u, c[0] + 2 * u, c[1] + 2 * u], fill=ink)
        d.ellipse([c[0] - 0.4 * u, c[1] - 1.4 * u, c[0] + 0.4 * u, c[1] - 0.6 * u], fill=(255, 255, 255))
    smile = [at(-4 + k * 8 / 12, -9 + ((k - 6) / 6) ** 2 * -2 + 2) for k in range(13)]
    d.line(smile, fill=ink, width=int(1.5 * u), joint="curve")
    for s in (-1, 1):
        c = at(s * 10, -11)
        d.ellipse([c[0] - 2.2 * u, c[1] - 2.2 * u, c[0] + 2.2 * u, c[1] + 2.2 * u], fill=(255, 140, 140, 110))
    return img.resize((S, S), Image.LANCZOS)


def main():
    OUT.mkdir(exist_ok=True)
    icon = draw()
    icon.save(OUT / "icon.png")
    icon.save(OUT / "matheo.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    if shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as tmp:
            iconset = Path(tmp) / "Matheo.iconset"
            iconset.mkdir()
            for size in (16, 32, 128, 256, 512):
                icon.resize((size, size), Image.LANCZOS).save(iconset / f"icon_{size}x{size}.png")
                icon.resize((size * 2, size * 2), Image.LANCZOS).save(iconset / f"icon_{size}x{size}@2x.png")
            subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(OUT / "Matheo.icns")], check=True)


if __name__ == "__main__":
    main()
