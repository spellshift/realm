extern crate golem;
extern crate eldritch;

use clap::{Command, arg};
use std::fs::File;
use std::path::Path;

// async fn install(config_path: String) -> Result<(), imix::Error> {
//     let config_file = File::open(config_path)?;
//     let config: imix::Config = serde_json::from_reader(config_file)?;

//     #[cfg(target_os = "windows")]
// 	    return imix::windows::install(config).await;
	
//     #[cfg(target_os = "linux")]
//         if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
//             return imix::linux::install(config).await;
//         }
	
// 	unimplemented!("The current OS/Service Manager is not supported")
// }

// async fn run(config_path: String) -> Result<(), imix::Error> {
//     let config_file = File::open(config_path)?;
//     let config: imix::Config = serde_json::from_reader(config_file)?;

//     #[cfg(target_os = "windows")]
//         return imix::windows::run(config).await;

//     #[cfg(target_os = "linux")]
//         if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
//             return imix::linux::run(config).await;
//         }

//     unimplemented!("The current OS/Manager is not supported")
// }

#[tokio::main]
async fn main() -> Result<(), golem::Error> {
    // let matches = Command::new("imix")
    //     .arg(
    //         arg!(
    //             -f --file <FILE> "Execute a specific tome file"
    //         )
    //         .required(false)
    //     )
    //     .get_matches();
    Ok(())
    // if let Some(tome_path) = matches.value_of("file") {
    //     return run(String::from(tome_path)).await
    // }

    // match matches.subcommand() {
    //     Some(("install", args)) => {
    //         let config_path = args.value_of("config").unwrap();
    //         install(String::from(config_path)).await
    //     },
    //     _ => Ok(())
    // }
}
