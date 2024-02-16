fn main() {
    tonic_build::configure()
    .out_dir("src/connection_manager")
    .include_file("mod.rs")
    .compile(
        &["src/proto/connection_manager.proto"],
        &["proto"]
    )
    .unwrap();
}