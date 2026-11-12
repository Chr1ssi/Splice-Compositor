"""keyboard-shortcuts-inhibit and xdg-activation (urgent windows) against the nested compositor.

Needs `cargo build --workspace` (crates/splice-test-client). Run from the repository root with
$DISPLAY set: PYTHONPATH=tests python3 tests/attention_smoke.py
"""
import os, subprocess, tempfile, time
from smoke_support import Compositor, file_pixel, pixel, wait

CLIENT = "./target/debug/splice-test-client"


def windows(comp):
    return open(comp.dir + "/log").read().count("new window")


# --- A client takes the shortcuts; release_shortcuts takes them back.
flag = tempfile.mktemp()
top = f'terminal = ["touch", "{flag}"]\n'
comp = Compositor(top=top, extra_args=f"{CLIENT} inhibit 40 & sleep 1; ")
assert wait(lambda: windows(comp) == 1, 10)
time.sleep(1.5)
comp.key("alt+Return")  # Super+Return: the terminal binding
time.sleep(1)
assert not os.path.exists(flag), "a binding ran although the client inhibits shortcuts"
comp.key("alt+shift+Escape")  # release_shortcuts
time.sleep(0.5)
comp.key("alt+Return")
assert wait(lambda: os.path.exists(flag), 5), "the binding does not work after release_shortcuts"
assert comp.stop()
print("inhibit OK")

# --- A window that asks for attention without user input gets the urgent border.
comp = Compositor(extra_args=f"{CLIENT} urgent 40 & sleep 1; ")
assert wait(lambda: windows(comp) == 2, 10)
assert wait(lambda: pixel(comp, 9, 400) == b"\xf3\x8b\xa8", 10), "the urgent border does not show"
shot = f"{comp.dir}/urgent.png"
subprocess.run(["grim", shot], env=comp.env(), check=True, timeout=15)
# Two columns: the first (unfocused, urgent) starts at x=8 with a 2px border.
assert file_pixel(shot, 9, 400) == "#F38BA8", file_pixel(shot, 9, 400)
assert comp.stop()
print("urgent OK")
print("OK")
