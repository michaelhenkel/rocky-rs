fn main() {
    tonic_build::configure()
    .out_dir("src/connection_manager")
    .include_file("mod.rs")
    .compile(
        &["src/proto/connection_manager.proto"],
        &["proto"]
    )
    .unwrap();

    tonic_build::configure()
    .out_dir("monitor/src/server")
    .include_file("mod.rs")
    .compile(
        &["monitor/src/proto/monitor.proto"],
        &["proto"]
    )
    .unwrap();
}