use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./embedded"]
pub struct Asset;
