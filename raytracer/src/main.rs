fn main() {
    if let Err(error) = raytracer::run() {
        eprintln!("raytracer: {error}");
        std::process::exit(1);
    }
}
