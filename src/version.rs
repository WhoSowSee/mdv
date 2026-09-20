use std::io::{self, Write};

pub(crate) fn print() -> io::Result<()> {
    writeln!(
        io::stdout().lock(),
        "mdv\n    Version: {}\n    Debug  : {}\n    Triple : {} ({}-{})\n    Rustc  : {}",
        env!("MDV_BUILD_VERSION"),
        cfg!(debug_assertions),
        env!("MDV_BUILD_TARGET"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("MDV_BUILD_RUSTC"),
    )
}
