//! The only reason for this helper's existence is that `headless-chrome` does not
//! allow us to pass arbitrary commandline-switches to chrome, which we need to
//! do in CI

use std::{env, io, process};

fn main() -> io::Result<()> {
    let args = env::args_os().skip(1);
    let status = process::Command::new("google-chrome")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .arg("--disable-dev-shm-usage")
        .arg("--start-maximized")
        .args(args)
        .status();
    match status {
        Ok(code) => {
            if code.success() {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Chrome failed with exit status {code}"),
                ))
            }
        }
        Err(e) => Err(e),
    }
}
