"""An X11 window that asks for fullscreen before it is mapped, the way SDL does, for xwayland_smoke.py.

Prints whether every size it was given is the screen's: a game that sizes its picture once keeps the first one.
Xlib through ctypes: the smoke shell has libX11 (LD_LIBRARY_PATH) but no Python X bindings.
"""
import ctypes, time
from ctypes import POINTER, Structure, Union, byref, c_char_p, c_int, c_long, c_uint, c_ulong, c_void_p

CONFIGURE_NOTIFY, STRUCTURE_NOTIFY_MASK, XA_ATOM, REPLACE = 22, 1 << 17, 4, 0


class XClassHint(Structure):
    _fields_ = [("res_name", c_char_p), ("res_class", c_char_p)]


class XConfigureEvent(Structure):
    _fields_ = [("type", c_int), ("serial", c_ulong), ("send_event", c_int), ("display", c_void_p), ("event", c_ulong),
                ("window", c_ulong), ("x", c_int), ("y", c_int), ("width", c_int), ("height", c_int)]


class XEvent(Union):
    _fields_ = [("type", c_int), ("configure", XConfigureEvent), ("pad", c_long * 24)]


x = ctypes.CDLL("libX11.so.6")
x.XOpenDisplay.argtypes = [c_char_p]
x.XOpenDisplay.restype = c_void_p
x.XDefaultRootWindow.argtypes = [c_void_p]
x.XDefaultRootWindow.restype = c_ulong
x.XDisplayWidth.argtypes = x.XDisplayHeight.argtypes = [c_void_p, c_int]
x.XCreateSimpleWindow.argtypes = [c_void_p, c_ulong, c_int, c_int, c_uint, c_uint, c_uint, c_ulong, c_ulong]
x.XCreateSimpleWindow.restype = c_ulong
x.XSetClassHint.argtypes = [c_void_p, c_ulong, POINTER(XClassHint)]
x.XInternAtom.argtypes = [c_void_p, c_char_p, c_int]
x.XInternAtom.restype = c_ulong
x.XChangeProperty.argtypes = [c_void_p, c_ulong, c_ulong, c_ulong, c_int, c_int, POINTER(c_ulong), c_int]
x.XSelectInput.argtypes = [c_void_p, c_ulong, c_long]
x.XMapWindow.argtypes = [c_void_p, c_ulong]
x.XPending.argtypes = [c_void_p]
x.XNextEvent.argtypes = [c_void_p, POINTER(XEvent)]

display = x.XOpenDisplay(None)
assert display, "no X display"
screen = (x.XDisplayWidth(display, 0), x.XDisplayHeight(display, 0))
window = x.XCreateSimpleWindow(display, x.XDefaultRootWindow(display), 0, 0, 200, 100, 0, 0, 0)
x.XSetClassHint(display, window, byref(XClassHint(b"fullscreenfirst", b"FullscreenFirst")))
fullscreen = c_ulong(x.XInternAtom(display, b"_NET_WM_STATE_FULLSCREEN", 0))
x.XChangeProperty(display, window, x.XInternAtom(display, b"_NET_WM_STATE", 0), XA_ATOM, 32, REPLACE, byref(fullscreen), 1)
x.XSelectInput(display, window, STRUCTURE_NOTIFY_MASK)
x.XMapWindow(display, window)

sizes = []
event = XEvent()
end = time.time() + 3
while time.time() < end:
    if not x.XPending(display):
        time.sleep(0.01)
        continue
    x.XNextEvent(display, byref(event))
    # The high bit marks an event the window manager sent itself (the synthetic ConfigureNotify).
    if event.type & 0x7F == CONFIGURE_NOTIFY:
        sizes.append((event.configure.width, event.configure.height))
ok = sizes and all(size == screen for size in sizes)
print(f"fullscreen-first {'OK' if ok else 'FAILED'}: screen {screen}, sizes {sizes}", flush=True)
time.sleep(60)
