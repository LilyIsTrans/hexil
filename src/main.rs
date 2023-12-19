mod app;
mod logging;
mod render;
mod window;

fn main() {
    use render::*;
    use tokio::runtime as rt;
    use window::*;
    let _guard = logging::init_tracing_to_file();
    let tok = rt::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _tok_guard = tok.enter();

    let eloop = make_event_loop().unwrap();
    let window = std::sync::Arc::new(make_window("Hexil", &eloop).unwrap());
    let (render_command_sender, render_rec) = tokio::sync::mpsc::channel::<RenderCommand>(100);
    let render_thread = tok.spawn(render_thread(window.clone(), render_rec));
    let eprox = eloop.create_proxy();

    run_event_loop(eloop).unwrap();
}
