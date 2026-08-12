fn main() {
    if let Err(e) = ctx::cli::run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
