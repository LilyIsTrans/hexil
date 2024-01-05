use build_time::build_time_local;

use tracing::info;

use tracing_appender::non_blocking::WorkerGuard;

use tracing_subscriber::fmt::format::FmtSpan;
/// ## Initialize logging facilities
/// This should be called exactly once, as early as possible, in the main thread.
/// Initializes the global logging facilities. Logs generated before this function runs
/// will never be recorded.
///
/// As long as the returned `WorkerGuard` is not dropped, all logs generated are guaranteed
/// to eventually be recorded to the log file, even in the event of a panic before they are
/// logged. All logs which have not been recorded yet when the returned `WorkerGuard` is dropped
/// will be immediately logged as part of the `WorkerGuard`'s `Drop` implementation.
pub fn init_tracing_to_file() -> WorkerGuard {
    use tracing_appender as ta;
    let filer = ta::rolling::Builder::new()
        .rotation(ta::rolling::Rotation::MINUTELY)
        .filename_prefix(env!("CARGO_PKG_NAME").to_string())
        .filename_suffix("log")
        .max_log_files(20)
        .build("logs")
        .unwrap();

    let (ace_writer, guard) = ta::non_blocking(filer);
    let sub = tracing_subscriber::fmt()
        .with_target(true)
        .with_ansi(true)
        .with_file(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_line_number(true)
        .with_level(true)
        .pretty()
        .with_writer(ace_writer);
    sub.init();
    info!("Hexil Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Built: {}", build_time_local!("%Y-%b-%d-%r-%s"));
    info!("Commit: {}", env!("GIT_HASH"));

    guard
}
