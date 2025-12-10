use alloc::collections::BTreeMap;
use alloc::string::ToString;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../imix/install_scripts"]
pub struct Asset;

pub fn get_embedded_assets() -> BTreeMap<String, Vec<u8>> {
    let mut assets = BTreeMap::new();
    for file in Asset::iter() {
        let name = file.as_ref().to_string();
        if let Some(content) = Asset::get(&name) {
            assets.insert(name, content.data.to_vec());
        }
    }
    assets
}
