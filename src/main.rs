#![windows_subsystem = "windows"]

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
use tracing::error;

fn main() {
    use render::*;
    use window::*;
    let _guard = logging::init_tracing_to_file();

    let eloop = make_event_loop().unwrap();
    let window = std::sync::Arc::new(make_window("Hexil", &eloop).unwrap());
    let (render_command_sender, render_rec) = std::sync::mpsc::channel::<RenderCommand>();
    let win = window.clone();
    let render_thread = std::thread::spawn(|| render_thread(win, render_rec));
    let _eprox = eloop.create_proxy();
    run_event_loop(eloop, render_command_sender.clone()).unwrap();
    if let Err(e) = render_thread.join() {
        error!("Render thread join error: {:#?}", e);
    }
}
