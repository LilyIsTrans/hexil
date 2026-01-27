use std::backtrace::Backtrace;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum HexilError {
    #[error(
        "The platform is unsupported. To be clear, we didn't just look at the platform and shut down on principle; we tried to create a window and found that we don't know how to do that here."
    )]
    UnsupportedPlatform(Backtrace),
}
