use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./install_scripts"]
pub struct Asset;
