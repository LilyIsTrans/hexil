use build_time::build_time_local;

use tracing::info;

use tracing_appender::non_blocking::WorkerGuard;

use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_tracing_to_file() -> WorkerGuard {
    use tracing_appender as ta;
    let filer_verbose = ta::rolling::Builder::new();
    let filer_verbose = filer_verbose.rotation(ta::rolling::Rotation::HOURLY);
    let filer_verbose = filer_verbose.filename_prefix(format!("{}", env!("CARGO_PKG_NAME")));
    let filer_verbose = filer_verbose.filename_suffix("log.verbose");
    let filer_verbose = filer_verbose.max_log_files(20);
    let filer_verbose = filer_verbose.build("logs").unwrap();

    // let filer_terse = ta::rolling::Builder::new();
    // let filer_terse = filer_terse.rotation(ta::rolling::Rotation::HOURLY);
    // let filer_terse = filer_terse.filename_prefix(format!(
    //     "{}-version-{}-built-{}-commit-{}-logged-at-",
    //     env!("CARGO_PKG_NAME"),
    //     env!("CARGO_PKG_VERSION"),
    //     build_time_local!("%Y-%b-%d-%r-%s"),
    //     env!("GIT_HASH"),
    // ));
    // let filer_terse = filer_terse.filename_suffix("log.verbose");
    // let filer_terse = filer_terse.max_log_files(20);
    // let filer_terse = filer_terse.build("").unwrap();

    let (ace_writer, guard) = ta::non_blocking(filer_verbose);
    let sub = tracing_subscriber::fmt();
    let sub = sub.with_target(cfg!(debug_assertions));
    let sub = sub.with_ansi(true);
    let sub = sub.with_file(true);
    let sub = sub.with_span_events(FmtSpan::FULL);
    let sub = sub.with_line_number(true);
    let sub = sub.with_level(true);
    let sub = sub.pretty();
    let sub = sub.with_writer(ace_writer);
    sub.init();
    info!("Hexil Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Built: {}", build_time_local!("%Y-%b-%d-%r-%s"));
    info!("Commit: {}", env!("GIT_HASH"));

    guard
}
