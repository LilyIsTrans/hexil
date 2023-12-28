/// Contains the completely implementation-agnostic code, primarily dealing with project files and device independant colours.
pub mod app;
/// Separates out the logging initialization to it's own file. There's only one function here.
pub mod logging;
/// Contains the rendering code. Currently, the renderer only supports Vulkan. Ideally, `render_thread` should be run in a dedicated
/// OS thread.
pub mod render;
/// Contains the windowing code. Currently this is handled with `winit`, but in the future it might contain platform
/// dependent code. `run_event_loop` must be called from the main thread, for compatibility with certain platforms `winit`
/// supports that we don't.
pub mod window;
