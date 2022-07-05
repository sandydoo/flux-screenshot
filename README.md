# Flux Screenshot

> Right now, Flux v4 can’t render to the framebuffer created here. You need to modify `flux.rs` to save a handle to the currently attached framebuffer *before* running the fluid compute pass, and then bind it right before rendering. Eventually, this will be fixed 🙃

Save screenshots of Flux at any resolution and point in time. “Screenshot” is a misnomer though. This takes advantage of headless OpenGL, drawing to a renderbuffer without opening a window. Loosely based on one of the examples from the glutin library. Tested exclusively on macOS.

## Sample

A headless render of Flux running at a logical resolution of 1280x800px and a 2.625 scaling factor, for a final physical resolution of 3360x2100px.

![A render of Flux](samples/flux-at-1280-800-logical.webp)

