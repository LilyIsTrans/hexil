use build_time::build_time_local;
use tracing::error;
use tracing::instrument;
use tracing_log::LogTracer;

#[instrument]
pub(crate) fn init_log_compat_layer() {
    if let Err(e) = LogTracer::init() {
        error!("Failed to initialize log crate compatibility layer: {}", e);
    }
}

pub(crate) fn init_tracing_to_file() {
    use tracing_appender as ta;
    let filer = ta::rolling::Builder::new();
    let filer = filer.rotation(ta::rolling::Rotation::HOURLY);
    let filer = filer.filename_prefix(format!(
        "{}-version-{}-built-{}-log-for-",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        build_time_local!("%Y-%b-%d-%r-%s")
    ));
}
