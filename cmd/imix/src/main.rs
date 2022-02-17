extern crate imix;

use clap::{App, arg};
use std::fs::File;
use std::path::Path;


async fn install(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(target_os = "windows")]
	    return imix::windows::install(config).await;
	
    #[cfg(target_os = "linux")]
        if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
            return imix::linux::install(config).await;
        }
	
	unimplemented!("The current OS/Service Manager is not supported")
}

async fn run(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(target_os = "windows")]
        return imix::windows::run(config).await;

    #[cfg(target_os = "linux")]
        if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
            return imix::linux::run(config).await;
        }

    unimplemented!("The current OS/Manager is not supported")
}

#[tokio::main]
async fn main() -> Result<(), imix::Error> {
    let matches = App::new("imix")
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false)
        )
        .subcommand(
            App::new("install")
                .about("Run in install mode")
                .arg(
                    arg!(
                        -c --config <FILE> "Sets a custom config file"
                    )
                    .required(true)
                )
        )
        .get_matches();
    
    if let Some(config_path) = matches.value_of("config") {
        return run(String::from(config_path)).await
    }

    match matches.subcommand() {
        Some(("install", args)) => {
            let config_path = args.value_of("config").unwrap();
            install(String::from(config_path)).await
        },
        _ => Ok(())
    }
}
