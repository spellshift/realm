use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "../imix/install_scripts"]
pub struct Asset;

impl eldritch_libassets::RustEmbed for Asset {
    fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
        <Asset as rust_embed::RustEmbed>::get(file_path)
    }

    fn iter() -> impl Iterator<Item = Cow<'static, str>> {
        <Asset as rust_embed::RustEmbed>::iter()
    }
}
