"""Screen capture (wlr-screencopy) with grim against the nested compositor.

Needs grim and ImageMagick. Run from the repository root with $DISPLAY set:
    PYTHONPATH=tests python3 tests/screencopy_smoke.py
"""
import subprocess
from smoke_support import Compositor, file_pixel, pixel, wait


def size(path):
    return subprocess.check_output(["identify", "-format", "%wx%h", path]).decode()


comp = Compositor(windows=2)
# The windows are drawn once their middle is no longer the palette background.
assert wait(lambda: pixel(comp, 320, 400) != bytes.fromhex("1E1E2E"), 15), "the windows were not drawn"
env = comp.env()
full, part, cursor = (f"{comp.dir}/{name}.png" for name in ("full", "part", "cursor"))
subprocess.run(["grim", full], env=env, check=True, timeout=15)
assert size(full) == "1280x800", size(full)
# The gap around the tiles shows the palette background; the middle of a window does not.
assert file_pixel(full, 2, 2) == "#1E1E2E", file_pixel(full, 2, 2)
assert file_pixel(full, 320, 400) != "#1E1E2E"
# A region.
subprocess.run(["grim", "-g", "100,100 200x150", part], env=env, check=True, timeout=15)
assert size(part) == "200x150", size(part)
# With the cursor overlaid (a different request flag).
subprocess.run(["grim", "-c", cursor], env=env, check=True, timeout=15)
assert size(cursor) == "1280x800"
assert comp.stop()
print("OK")
