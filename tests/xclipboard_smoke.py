"""Copy and paste between Wayland and X11 clients (Xwayland), both directions.

Needs wl-clipboard, xclip, Xwayland and xterm. Run from the repository root with $DISPLAY set:
    PYTHONPATH=tests python3 tests/xclipboard_smoke.py
"""
import re, subprocess, time
from smoke_support import Compositor, wait


def poll_output(cmd, env, expected, timeout=15):
    """Run `cmd` until its stdout is `expected` (or the timeout passes); returns the last run's result."""
    result = None

    def done():
        nonlocal result
        result = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=15)
        return result.stdout == expected

    wait(done, timeout)
    return result


comp = Compositor(extra_args="xterm -e 'sleep 60' & ")
log = lambda: open(comp.dir + "/log").read()
assert wait(lambda: 'app_id=Some("XTerm")' in log(), 15), "no X11 window"
xdisplay = re.search(r"Xwayland on DISPLAY=(:\d+)", log()).group(1)
xenv = dict(comp.env(), DISPLAY=xdisplay)
time.sleep(1)

for selection, wl_flags in (("clipboard", []), ("primary", ["--primary"])):
    # Wayland -> X11
    subprocess.run(["wl-copy", *wl_flags, f"from wayland {selection}"], env=comp.env(), check=True, timeout=15)
    out = poll_output(["xclip", "-o", "-t", "UTF8_STRING", "-selection", selection], xenv, f"from wayland {selection}")
    assert out.stdout == f"from wayland {selection}", (selection, "wayland->x11", out.stdout, out.stderr)
    # X11 -> Wayland
    owner = subprocess.Popen(["xclip", "-i", "-t", "UTF8_STRING", "-selection", selection, "-loops", "3"], env=xenv, stdin=subprocess.PIPE, text=True)
    owner.stdin.write(f"from x11 {selection}")
    owner.stdin.close()
    out = poll_output(["wl-paste", "--no-newline", *wl_flags], comp.env(), f"from x11 {selection}")
    assert out.stdout == f"from x11 {selection}", (selection, "x11->wayland", out.stdout, out.stderr)
    owner.kill()
    print(selection, "both ways OK")
assert comp.stop()
print("OK")
