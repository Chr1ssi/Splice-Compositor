"""A floating window starts at the size of its first buffer, not at the layout's default size.

Needs `cargo build --workspace`. Run from the repository root with $DISPLAY set:
    PYTHONPATH=tests python3 tests/floating_size_smoke.py
"""
import subprocess
from smoke_support import Compositor, wait

CLIENT = "./target/debug/splice-test-client"
BORDER = {"#89B4FA", "#45475A"}


def row(path, y, xs):
    """Colors of the pixels at `xs` in row `y`."""
    out = subprocess.check_output(["convert", path, "-crop", "1280x1+0+%d" % y, "txt:-"]).decode()
    colors = {}
    for line in out.splitlines()[1:]:
        position, rest = line.split(":", 1)
        colors[int(position.split(",")[0])] = rest.split()[1].upper()
    return [colors[x] for x in xs]


# The keys client draws a fixed 200x150 buffer whatever size it is configured to, like a GTK dialog.
comp = Compositor('[[rules]]\napp_id = "splice.test.0"\nfloating = true\n')
subprocess.Popen([CLIENT, "keys", "30"], env=comp.env(), stdout=subprocess.DEVNULL)
assert wait(lambda: open(comp.dir + "/log").read().count("new window") >= 1, 10)
path = f"{comp.dir}/shot.png"


def bordered():
    subprocess.run(["grim", path], env=comp.env(), check=True, timeout=15)
    return sum(p in BORDER for p in row(path, 400, range(520, 541)) + row(path, 400, range(740, 761))) >= 2


# Waits until the frame is drawn around the content. On timeout the assertions below report what is on screen.
wait(bordered, 10)
subprocess.run(["grim", path], env=comp.env(), check=True, timeout=15)
# Centered on the 1280x800 screen, the content spans x 540..740; the border hugs it.
near = row(path, 400, range(520, 541)) + row(path, 400, range(740, 761))
assert sum(p in BORDER for p in near) >= 2, f"no border around the window's content: {sorted(set(near))}"
far = row(path, 400, list(range(100, 500)) + list(range(780, 1180)))
assert not any(p in BORDER for p in far), "the frame is larger than the window's content"
assert comp.stop()
print("floating window at its client's size OK")
print("OK")
