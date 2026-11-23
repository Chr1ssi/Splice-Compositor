# Vendored dependencies

## smithay master `a38bb709` (patched)

`vendor/smithay` is smithay's git master at commit `a38bb709` (2026-11-19; the crate still calls itself
0.7.0, the latest release, from 2025-06), packaged the way crates.io would ship it
(`cargo package -p smithay` in a checkout of that commit), plus the changes in `smithay.patch`:

- `X11Surface::set_x11_input_focus`: the X11 half of `KeyboardTarget::enter/leave`. Our keyboard
  focus is a plain `wl_surface`, so without it Xwayland never gets an X11 input focus and Wine/Proton
  games (which wait for `WM_TAKE_FOCUS`) receive no keys.
- `X11Surface::set_hidden` flushes the X connection after it wrote `WM_STATE`. Wine minimizes an
  exclusive-fullscreen window when it loses the focus and only follows a restore that is reported
  through `WM_STATE`, which otherwise waited in the buffer for the next X event.
- `X11Wm::new_selection` flushes the X connection. The owner change otherwise stays buffered until
  another X event arrives, so a freshly copied Wayland selection was invisible to X11 clients.
- `X11Sockets` and `XWayland::spawn_with_sockets`: the display slot and listening sockets can be opened first
  (`X11Sockets::prepare`) and handed to Xwayland later, so that the compositor starts it when the first X11
  client connects (the connection waits in the socket's backlog). `XWayland::spawn` is now `prepare` plus
  the private spawn.
- `#![allow(warnings)]` in `src/lib.rs` so the vendored code does not clutter our build.

Upstream since took over two earlier patches: `set_hidden` replaced our `set_iconic`, and the map request reads
the `_NET_WM_STATE` the client set before mapping (our `read_initial_net_state`).

## smithay-drm-extras from the same commit (unpatched)

`vendor/smithay-drm-extras` is `smithay-drm-extras` from the same commit, packaged the same way
(`cargo package -p smithay-drm-extras`), without changes. It is still 0.1.0 like the crates.io release, but has
the `DrmScanner` hotplug fixes since: CRTC reservations of vanished connectors are released, connectors that
disappear from the resource list are reported as unplugged, and a new `DrmScanEvent::Changed` reports a changed
mode list of a connected display (Splice logs it and keeps the mode).

The root `Cargo.toml` redirects `smithay` and `smithay-drm-extras` here with `[patch.crates-io]`.

To drop a change once smithay supports it upstream, adjust `smithay.patch`; to go back to crates.io, delete the
`[patch.crates-io]` section and this directory.

To check what differs from upstream: package that commit the same way, unpack the `.crate` next to
`vendor/smithay` and run `diff -ru smithay-0.7.0 vendor/smithay`.
