mod app;
mod logging;
mod render;
mod window;

fn main() {
    use tokio::runtime as rt;
    use window::*;
    let _guard = logging::init_tracing_to_file();

    let eloop = make_event_loop().unwrap();
    let window = make_window("Hexil", &eloop);
    let eprox = eloop.create_proxy();

    run_event_loop(eloop).unwrap();
}
