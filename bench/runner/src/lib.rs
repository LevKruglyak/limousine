use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
pub use serde_json::from_str;

pub const TEMP_STORAGE_PATH: &str = ".limousine_benchmarks";
pub const LIMOUSINE_INSTANCE_PATH: &str = "bench/instance";
pub const LIMOUSINE_INSTANCE_CONFIG: &str = ".config";

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceParams {
    pub key_type: String,
    pub value_size: usize,
    pub size: usize,
    pub path: PathBuf,
    pub layout: String,
}

/// Get the current path to current workspace
pub fn get_current_workspace() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
