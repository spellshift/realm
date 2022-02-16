extern crate imix;

use clap::{App, arg};
use std::fs::File;
use std::path::Path;


fn install(file_path: String) -> std::io::Result<()> {
    println!("Installing with {} config...", file_path);
    let config_file = File::open(file_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    if cfg!(windows) {
        return imix::windows::install(config);
    }

    if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
        return imix::linux::install(config);
    }

    unimplemented!("The current OS/Service Manager is not supported")
}

fn run() -> std::io::Result<()> {
    println!("Running...");

    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("imix")
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
    
    match matches.subcommand() {
        Some(("install", args)) => {
            let file_path_str = args.value_of("config").unwrap();
            install(String::from(file_path_str))
        },
        _ => run(),
    }
}
