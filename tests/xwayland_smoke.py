"""X11 clients through Xwayland, and switching Xwayland off.

Needs Xwayland, xterm, xeyes and libX11. Run from the repository root with $DISPLAY set:
    PYTHONPATH=tests python3 tests/xwayland_smoke.py
"""
import os
import time
from smoke_support import Bar, Compositor, wait


def log(comp):
    return open(comp.dir + "/log").read()


comp = Compositor(extra_args="xterm -e 'sleep 60' & xeyes & ")
try:
    bar = Bar(comp.sock)
    assert wait(lambda: 'app_id=Some("XTerm")' in log(comp) and 'app_id=Some("XEyes")' in log(comp), 15), \
        "X11 windows were not managed"
    bar.until(lambda s: s[0]["workspaces"].get(1))
    assert "X11 window manager ready" in log(comp)
    # Two X11 windows tile like any others: three would scroll, two fit exactly.
    time.sleep(1)
    bar.poll()
    assert not bar.state[0]["left"] and not bar.state[0]["right"], bar.state
    assert comp.stop()
    print("Xwayland clients OK")
except BaseException:
    comp.process.kill()
    print(log(comp)[-3000:])
    raise

# WM_TRANSIENT_FOR is the client's to set: a window transient for itself is just a floating window.
looped = Compositor(extra_args="python3 tests/x11_self_transient.py & ")
try:
    assert wait(lambda: 'app_id=Some("SelfTransient")' in log(looped), 15), "the window was not managed"
    time.sleep(0.5)
    assert looped.process.poll() is None, "the compositor died"
    assert looped.stop()
    print("window transient for itself OK")
except BaseException:
    looped.process.kill()
    print(log(looped)[-3000:])
    raise

# Xwayland runs only once an X11 client connects: the sockets exist from the start, the server does not.
lazy = Compositor()
try:
    import re
    import subprocess
    assert wait(lambda: "Xwayland starts with the first X11 client" in log(lazy), 15), "no on-demand start announced"
    time.sleep(1)
    spawned = lambda: subprocess.run(["pgrep", "-P", str(lazy.process.pid), "-x", "Xwayland"], capture_output=True).returncode == 0
    assert not spawned(), "Xwayland started without a client"
    number = re.search(r"on DISPLAY=:(\d+)", log(lazy)).group(1)
    client = subprocess.Popen(["xterm", "-e", "sleep 30"], env=dict(os.environ, DISPLAY=f":{number}"))
    assert wait(lambda: 'app_id=Some("XTerm")' in log(lazy), 20), "a late X11 client was not managed"
    assert spawned(), "Xwayland is not running after its first client"
    assert "X11 primary output" in log(lazy), "the primary output was not set\n" + log(lazy)[-1500:]
    client.terminate()
    assert lazy.stop()
    print("Xwayland on demand OK")
except BaseException:
    lazy.process.kill()
    print(log(lazy)[-3000:])
    raise

off = Compositor(top="xwayland = false\n", extra_args="sleep 2 & ")
try:
    time.sleep(3)
    assert "Xwayland on" not in log(off), "xwayland = false was ignored"
    assert off.stop()
    print("xwayland = false OK")
    print("OK")
except BaseException:
    off.process.kill()
    print(log(off)[-3000:])
    raise
