fn main() {
    reset_sigpipe();
    if let Err(e) = ctx::cli::run() {
        eprintln!("error: {e}");
        let code = match &e {
            ctx::errors::CtxError::Usage(_) => 2,
            _ => 1,
        };
        std::process::exit(code);
    }
}

/// Rust's runtime ignores SIGPIPE, which turns `ctx | head` into a panicking
/// broken-pipe error. Restore the default so the process exits quietly, like
/// every other Unix CLI.
fn reset_sigpipe() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
