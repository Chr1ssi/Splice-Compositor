"""The exit binding ends the session at once while the compositor draws no shell that could ask first.

Needs `cargo build --workspace`. Run from the repository root with $DISPLAY set:
    PYTHONPATH=tests python3 tests/logout_smoke.py
"""
from smoke_support import Bar, Compositor, wait

# Nested, Super is Alt: the default exit binding Super+m is Alt+m here.
comp = Compositor()
try:
    bar = Bar(comp.sock)
    bar.until(lambda s: len(s) == 1)
    comp.key("alt+m")
    assert wait(lambda: comp.process.poll() is not None), "the exit binding did not end the session"
    print("exit binding without a shell ends the session OK")
except BaseException:
    comp.process.kill()
    print(open(comp.dir + "/log").read()[-3000:])
    raise
print("OK")
