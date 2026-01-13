use eldritch_macros::EncryptedEmbed;
use eldritchv2::assets::std::Embedable;

#[derive(EncryptedEmbed)]
#[folder = "../imix/install_scripts"]
pub struct Asset;
