mod app;
mod logging;
mod render;
mod window;

fn main() {
    use render::*;
    use window::*;
    let _guard = logging::init_tracing_to_file();

    let eloop = make_event_loop().unwrap();
    let window = std::sync::Arc::new(make_window("Hexil", &eloop).unwrap());
    let (render_command_sender, render_rec) = std::sync::mpsc::channel::<RenderCommand>();
    let win = window.clone();
    let render_thread = std::thread::spawn(|| render_thread(win, render_rec));
    let eprox = eloop.create_proxy();
    run_event_loop(eloop, render_command_sender.clone()).unwrap();
}
