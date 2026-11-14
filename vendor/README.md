# Vendored dependencies

## smithay 0.7.0 (patched)

`vendor/smithay` is the unmodified crates.io release of smithay 0.7.0 plus the
changes in `smithay.patch`:

- `X11Surface::set_x11_input_focus`: the X11 half of `KeyboardTarget::enter/leave`. Our keyboard
  focus is a plain `wl_surface`, so without it Xwayland never gets an X11 input focus and Wine/Proton
  games (which wait for `WM_TAKE_FOCUS`) receive no keys.
- `X11Surface::set_iconic`: tells an X11 client it is minimized (`WM_STATE` iconic, `_NET_WM_STATE_HIDDEN`)
  or restored without unmapping the window, so it keeps its picture and its place. Wine minimizes an
  exclusive-fullscreen window when it loses the focus and only follows a restore that is reported
  through `WM_STATE`.
- `X11Wm::new_selection` flushes the X connection. The owner change otherwise stays buffered until
  another X event arrives, so a freshly copied Wayland selection was invisible to X11 clients.
- `X11Sockets` and `XWayland::spawn_with_sockets`: the display slot and listening sockets can be opened first
  (`X11Sockets::prepare`) and handed to Xwayland later, so that the compositor starts it when the first X11
  client connects (the connection waits in the socket's backlog). `XWayland::spawn` is now `prepare` plus
  `spawn_with_sockets`.
- `X11Surface::read_initial_net_state`, called on a map request: reads the `_NET_WM_STATE` the client set
  before mapping, so that `is_fullscreen()` and friends are true for a window that wants to open that way
  (EWMH; SDL does it). smithay only followed the client message, which a client may send once it is mapped.
- `#![allow(warnings)]` in `src/lib.rs` so the vendored code does not clutter our build.

The root `Cargo.toml` redirects `smithay` here with `[patch.crates-io]`.

To drop a change once smithay supports it upstream, adjust `smithay.patch`; to drop all of them, delete the
`[patch.crates-io]` section, `vendor/smithay` and this directory.

To check what differs from the release: unpack smithay 0.7.0 next to `vendor/smithay`
and run `diff -ru smithay-0.7.0 vendor/smithay`.
