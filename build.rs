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
    .type_attribute("MlxCounter", "#[path_resolver::derive_path::derive_path(path = \"/sys/class/infiniband/{{ interface }}/ports/{{ port }}/counters\")]")
    .type_attribute("MlxCounter", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("MlxHwCounter", "#[path_resolver::derive_path::derive_path(path = \"/sys/class/infiniband/{{ interface }}/ports/{{ port }}/hw_counters\")]")
    .type_attribute("MlxHwCounter", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("RxeCounter", "#[path_resolver::derive_path::derive_path(path = \"/sys/class/net/{{ linux_interface }}/statistics\")]")
    .type_attribute("RxeCounter", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("RxeHwCounter", "#[path_resolver::derive_path::derive_path(path = \"/sys/class/infiniband/{{ interface }}/ports/{{ port }}/hw_counters\")]")
    .type_attribute("RxeHwCounter", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("PerSec", "#[derive(serde::Deserialize, serde::Serialize)]")
    .type_attribute("BwResults", "#[derive(serde::Deserialize, serde::Serialize)]")
    .compile(
        &["monitor/src/proto/monitor.proto"],
        &["proto"]
    )
    .unwrap();
}