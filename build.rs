fn main() {
    tonic_build::configure()
    .out_dir("src/initiator/listener")
    .include_file("mod.rs")
    .compile(
        &["src/proto/listener.proto"],
        &["proto"]
    )
    .unwrap();

    tonic_build::configure()
    .out_dir("src/server/connection_manager")
    .include_file("mod.rs")
    .compile(
        &["src/proto/connection_manager.proto"],
        &["proto"]
    )
    .unwrap();
}