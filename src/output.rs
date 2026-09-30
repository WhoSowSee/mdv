use std::io::{self, Write};

pub(crate) fn write_stdout(text: &str) -> io::Result<()> {
    let mut output = io::stdout().lock();
    output.write_all(text.as_bytes())?;
    output.flush()
}
