mod logging;
mod render;
mod window;

fn main() {
    let _guard = logging::init_tracing_to_file();
}
