use std::{
    fs::File,
    io::{stdout, Write},
    process::{Command, Stdio},
    time::SystemTime,
};

use bench_common::{
    get_current_workspace, InstanceParams, LIMOUSINE_INSTANCE_CONFIG, LIMOUSINE_INSTANCE_PATH,
    TEMP_STORAGE_PATH,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum_macros::AsRefStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clean(CleanArgs),
    Bench(BenchArgs),
}

#[derive(Args)]
pub struct CleanArgs {
    args: CleanType,
}

#[derive(ValueEnum, Clone, Default, PartialEq)]
pub enum CleanType {
    Build,
    Data,
    Logs,
    #[default]
    All,
}

#[derive(ValueEnum, Clone, Copy, Debug, AsRefStr, Default)]
pub enum KeyType {
    #[default]
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
}

#[derive(Args, Debug)]
pub struct BenchArgs {
    #[arg(long, help = "Numeric key type.")]
    pub key_type: Option<KeyType>,

    #[arg(long, help = "Size of the values in bytes.", default_value_t = 0)]
    pub value_size: usize,

    #[arg(
        long,
        help = "Size of the key-value store prior to running inserts and gets.",
        default_value_t = 65536
    )]
    pub size: usize,

    #[arg(long, help = "Limousine layout.")]
    pub layout: String,
}

fn main() -> anyhow::Result<()> {
    // Get the current path to current workspace so that we have consistent absolute paths
    let current_workspace = get_current_workspace();

    let limousine_instance_path = current_workspace.join(LIMOUSINE_INSTANCE_PATH);
    let temp_storage_path = current_workspace.join(TEMP_STORAGE_PATH);

    let cli = Cli::parse();
    match &cli.command {
        Commands::Clean(args) => {
            // Clean temp data directory
            if temp_storage_path.exists() {
                let path = match args.args {
                    CleanType::All => Some(temp_storage_path),
                    CleanType::Data => Some(temp_storage_path.join("data")),
                    CleanType::Logs => Some(temp_storage_path.join("logs")),
                    _ => None,
                };

                if let Some(path) = path {
                    std::fs::remove_dir_all(path)?;
                }
            }

            if args.args == CleanType::All || args.args == CleanType::Build {
                // Clean the cargo build directory for limousine_instance
                Command::new("cargo")
                    .current_dir(limousine_instance_path)
                    .args(&["clean"])
                    .stdout(Stdio::null())
                    .spawn()?
                    .wait()?;
            }
        }
        Commands::Bench(args) => {
            // Create temp storage directory
            if !temp_storage_path.exists() {
                std::fs::create_dir(temp_storage_path.clone())?;
            }

            // Serialize the benchmarking arguments and put it into a file
            // for the build script to access
            let key_type = args.key_type.unwrap_or_default();
            let instance_params = InstanceParams {
                key_type: key_type.as_ref().to_string(),
                value_size: args.value_size,
                size: args.size,
                path: temp_storage_path.clone(),
                layout: args.layout.clone(),
            };

            let instance_json = serde_json::to_string(&instance_params)?;
            std::fs::write(
                limousine_instance_path.join(LIMOUSINE_INSTANCE_CONFIG),
                instance_json,
            )?;

            // Compile in release mode
            let time = humantime::format_rfc3339_seconds(SystemTime::now()).to_string();
            let temp_logs_path = temp_storage_path.join("logs");
            if !temp_logs_path.exists() {
                std::fs::create_dir(temp_logs_path.clone())?;
            }

            let log_path = temp_logs_path.join(format!("compile_{}.log", time));
            let err_path = temp_logs_path.join(format!("compile_{}.err", time));

            let log_file = File::create(log_path.clone())?;
            let err_file = File::create(err_path.clone())?;

            if !Command::new("cargo")
                .current_dir(limousine_instance_path.clone())
                .args(&["build", "--release", "--features", "instance"])
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(err_file))
                .spawn()?
                .wait()?
                .success()
            {
                eprint!("{}", std::fs::read_to_string(err_path)?);
                eprint!("Code generation failed! This is most likely an issue with the specified `layout`!");
                return Ok(());
            }

            print!("Generating code and compiling...    ");
            stdout().flush().unwrap();

            // Run in release mode
            let _ = Command::new(limousine_instance_path.join("target/release/limousine_instance"))
                .current_dir(limousine_instance_path)
                .spawn()?
                .wait()?;
        }
    }

    Ok(())
}
