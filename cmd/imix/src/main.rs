use clap::{App, arg};
use serde::{Serialize, Deserialize};
use std::fs::File;

#[derive(Serialize, Deserialize, Debug)]
struct C2Config {
    uri: String,
    timeout: u32,
    priority: u8,
    sticky: bool,
    failsafe: bool
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceConfig {
    name: String,
    description: String,
    executable_path: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    target_name: String,
    callback_interval: u32,
    callback_jitter: u32,
    c2_configs: Vec<C2Config>,
    service_configs: Vec<ServiceConfig>,
}


fn install(file_path: String) -> std::io::Result<()> {
    println!("Installing with {} config...", file_path);
    let config_file = File::open(file_path)?;
    let config: Config = serde_json::from_reader(config_file)?;
    println!("Loaded this: {:?}", config);

    Ok(())
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
