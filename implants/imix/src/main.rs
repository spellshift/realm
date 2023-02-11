extern crate imix;
extern crate eldritch;

use clap::{Command, arg};
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

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    unimplemented!("The current OS/Manager is not supported");

    Ok(imix::common::main_loop().await)

}

#[tokio::main]
async fn main() -> Result<(), imix::Error> {
    let matches = Command::new("imix")
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false)
        )
        .subcommand(
            Command::new("install")
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
