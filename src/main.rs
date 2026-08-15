fn main() {
    reset_sigpipe();
    if let Err(e) = ctx::cli::run() {
        let code = match &e {
            ctx::errors::CtxError::Usage(_) => 2,
            ctx::errors::CtxError::Unhealthy(_) => 1,
            _ => 1,
        };
        // `Unhealthy` already produced its full report (e.g. `ctx doctor`);
        // echoing `error:` again would be noise. Every other error is printed.
        if !matches!(e, ctx::errors::CtxError::Unhealthy(_)) {
            eprintln!("error: {e}");
        }
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
