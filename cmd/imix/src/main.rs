use clap::{App};

fn main() {
    let matches = App::new("imix")
        .subcommand(
            App::new("install")
                .about("Run in install mode")
        )
        .get_matches();
    
    match matches.subcommand() {
        Some(("install", _)) => println!("Installing..."),
        _ => println!("Running..."),
    }
}
