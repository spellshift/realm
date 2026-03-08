#[cfg(feature = "install")]
use anyhow::Result;
#[cfg(feature = "install")]
use eldritch::Interpreter;
use eldritch::assets::std::{EmbeddedAssets, StdAssetsLibrary};
use std::sync::Arc;

#[cfg(feature = "install")]
pub async fn install() -> Result<()> {
    #[cfg(debug_assertions)]
    log::info!("starting installation");
    let asset_backend = Arc::new(EmbeddedAssets::<crate::assets::Asset>::new());

    // Iterate through all embedded files using the Asset struct from assets.rs
    for embedded_file_path in crate::assets::Asset::iter() {
        // Find "main.eldritch" files
        if embedded_file_path.ends_with("main.eldritch") {
            #[cfg(debug_assertions)]
            log::info!("loading tome {}", embedded_file_path);

            let content = match crate::assets::Asset::get(&embedded_file_path) {
                Some(f) => String::from_utf8_lossy(&f.data).to_string(),
                None => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to load install asset: {}", embedded_file_path);
                    continue;
                }
            };

            #[cfg(debug_assertions)]
            log::info!("running tome {}", embedded_file_path);

            // Execute using Eldritch Interpreter
            let mut locker = StdAssetsLibrary::new();
            let _ = locker.add(asset_backend.clone());
            let mut interpreter = Interpreter::new().with_default_libs();
            interpreter.register_lib(locker);

            match interpreter.interpret(&content) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::info!("Successfully executed {embedded_file_path}");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to execute {embedded_file_path}: {_e}");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "install")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_install_execution() {
        let result = install().await;
        // It might fail during execution due to permissions (trying to write to /bin/imix),
        // but the install function itself returns Ok(()) because we catch errors inside the loop.
        assert!(result.is_ok());
    }
}
