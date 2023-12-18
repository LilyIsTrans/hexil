mod logging;
mod render;
mod window;

#[tokio::main]
async fn main() {
    let _guard = logging::init_tracing_to_file();
}
