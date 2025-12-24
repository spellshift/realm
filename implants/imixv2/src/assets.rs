use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../imix/install_scripts"]
pub struct Asset;
