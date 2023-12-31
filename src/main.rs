#![windows_subsystem = "windows"]

use hexil::logging;
use hexil::render;
use hexil::window;
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
