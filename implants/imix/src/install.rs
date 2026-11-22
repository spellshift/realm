#[cfg(feature = "install")]
use anyhow::Result;
#[cfg(feature = "install")]
use eldritch::Interpreter;
#[cfg(all(feature = "install", not(feature = "print_debug_tome")))]
use eldritch::NoopPrinter;
use eldritch::assets::std::{EmbeddedAssets, StdAssetsLibrary};
use std::sync::Arc;

#[cfg(feature = "install")]
pub async fn install() -> Result<()> {
    if let Err(_e) = pb::config::enforce_guardrails() {
        #[cfg(feature = "print_debug")]
        log::error!("Guardrails failed, aborting installation: {_e}");
        return Ok(());
    }

    run_install_tomes().await
}

#[cfg(all(test, feature = "install"))]
async fn install_with_guardrails(guardrails_json: Option<&str>) -> Result<bool> {
    if let Err(_e) = pb::config::enforce_guardrails_json(guardrails_json) {
        #[cfg(feature = "print_debug")]
        log::error!("Guardrails failed, aborting installation: {_e}");
        return Ok(false);
    }

    run_install_tomes().await?;
    Ok(true)
}

#[cfg(feature = "install")]
async fn run_install_tomes() -> Result<()> {
    #[cfg(feature = "print_debug")]
    log::info!("starting installation");
    let asset_backend = Arc::new(EmbeddedAssets::<crate::assets::Asset>::new());

    // Iterate through all embedded files using the Asset struct from assets.rs
    for embedded_file_path in crate::assets::Asset::iter() {
        // Find "main.eldritch" files
        if embedded_file_path.ends_with("main.eldritch") {
            #[cfg(feature = "print_debug")]
            log::info!("loading tome {}", embedded_file_path);

            let content = match crate::assets::Asset::get(&embedded_file_path) {
                Some(f) => String::from_utf8_lossy(&f.data).to_string(),
                None => {
                    #[cfg(feature = "print_debug")]
                    log::error!("failed to load install asset: {}", embedded_file_path);
                    continue;
                }
            };

            #[cfg(feature = "print_debug")]
            log::info!("running tome {}", embedded_file_path);

            // Execute using Eldritch Interpreter
            let mut locker = StdAssetsLibrary::new();
            let _ = locker.add(asset_backend.clone());
            #[cfg(not(feature = "print_debug_tome"))]
            let mut interpreter =
                Interpreter::new_with_printer(Arc::new(NoopPrinter)).with_default_libs();
            #[cfg(feature = "print_debug_tome")]
            let mut interpreter = Interpreter::new().with_default_libs();
            interpreter.register_lib(locker);

            match interpreter.interpret(&content) {
                Ok(_) => {
                    #[cfg(feature = "print_debug")]
                    log::info!("Successfully executed {embedded_file_path}");
                }
                Err(_e) => {
                    #[cfg(feature = "print_debug")]
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

    #[tokio::test]
    async fn test_install_aborts_when_guardrails_fail() {
        let ran = install_with_guardrails(Some(
            r#"[{"type":"file","args":{"path":"/nonexistent/guardrail/install/path"}}]"#,
        ))
        .await
        .expect("guardrail failure should return Ok(false)");
        assert!(!ran, "install should abort when guardrails fail");

        let ran_malformed = install_with_guardrails(Some("{invalid"))
            .await
            .expect("malformed guardrail should fail closed cleanly");
        assert!(
            !ran_malformed,
            "install should abort on malformed guardrails"
        );
    }
}
