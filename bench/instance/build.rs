use bench_common::{get_current_workspace, InstanceParams, LIMOUSINE_INSTANCE_CONFIG};

fn main() -> anyhow::Result<()> {
    // Read the config
    let config_path = get_current_workspace().join(LIMOUSINE_INSTANCE_CONFIG);
    let params: InstanceParams =
        bench_common::from_str(std::fs::read_to_string(config_path.clone())?.as_str())?;

    println!("cargo:rerun-if-changed={}", config_path.to_str().unwrap());
    println!("cargo:rustc-cfg=feature=\"instance\"");

    // Pass configs to cargo compilation
    println!("cargo:rustc-cfg=feature=\"key-{}\"", params.key_type);
    println!("cargo:rustc-env=VALUE_SIZE={}", params.value_size);
    println!("cargo:rustc-env=SIZE={}", params.size);
    println!(
        "cargo:rustc-env=STORE_PATH={}",
        params.path.to_str().unwrap()
    );

    // Write the layout file for the macro to read
    let layout_path = get_current_workspace().join(".layout");
    std::fs::write(layout_path, params.layout).unwrap();

    Ok(())
}
