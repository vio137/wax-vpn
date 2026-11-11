# Pinned source

Upstream: https://github.com/DGriffin91/egui_software_backend
Revision: c70e02d3d95c75a82733deb97e516b13a6c86dc6
Original version: 0.0.3, MIT OR Apache-2.0. Both license texts retained.
The constify helper is pinned to crates.io 0.0.1; its src/lib.rs was byte-compared
with this upstream revision and is identical. Source modules are inlined into one
lib.rs for distribution; module boundaries and cfg guards are preserved.

waX adaptation: egui/egui-winit 0.37 texture-delta API; consume deltas after
presentation; initialize AccessKit; preserve app state through suspend;
route close requests through egui CancelClose; honor delayed repaint deadlines,
including hidden-window tray polling; support native drag and edge resize.
Windows presentation uses softbuffer 0.4.8's GDI DIB/BitBlt backend. It does not
initialize a Direct3D, Vulkan or OpenGL renderer. Windows 10+ x64 is required.

This is not a full eframe replacement. waX uses one root window. Child viewports,
screenshots and several uncommon viewport commands remain unsupported. GPU
renderers remain available separately. No tunnel or routing code is changed.

Local UI launch/input/resize screenshots checked. Windows validation is required
before changing the startup default or publishing a compatibility claim. A hosted
Windows runner is not proof of a clean consumer Windows install or the reported PC.
