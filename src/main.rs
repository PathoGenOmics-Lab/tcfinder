fn main() {
    if let Err(e) = tcfinder::run() {
        println!("Application error: {e}");
        std::process::exit(1);
    }
}
