use anyhow::{anyhow, Result};
#[allow(unused_imports)]
use eldritch::runtime::{messages::AsyncMessage, Message};
use pb::eldritch::Tome;
use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::fmt::Write;

pub async fn install() {
    #[cfg(debug_assertions)]
    log::info!("starting installation");

    // Iterate through all embedded files
    for embedded_file_path in eldritch::assets::Asset::iter() {
        let filename = embedded_file_path.split('/').next_back().unwrap_or("");

        #[cfg(debug_assertions)]
        log::debug!("checking asset {embedded_file_path}");

        // Evaluate all "main.eldritch" files
        if filename == "main.eldritch" {
            // Read eldritch content from embedded file
            #[cfg(debug_assertions)]
            log::info!("loading tome {embedded_file_path}");
            let eldritch = match load_embedded_eldritch(embedded_file_path.to_string()) {
                Ok(content) => content,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to load install asset: {_err}");

                    continue;
                }
            };

            // Run tome
            #[cfg(debug_assertions)]
            log::info!("running tome {embedded_file_path}");
            let mut runtime = eldritch::start(
                0,
                Tome {
                    eldritch,
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
            )
            .await;
            runtime.finish().await;

            #[cfg(debug_assertions)]
            let mut output = String::new();

            #[cfg(debug_assertions)]
            for msg in runtime.collect() {
                if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                    if let Err(err) = output.write_str(m.text().as_str()) {
                        #[cfg(debug_assertions)]
                        log::error!("failed to write text: {}", err);
                    }
                }
            }
            #[cfg(debug_assertions)]
            log::info!("{output}");
        }
    }
}

fn load_embedded_eldritch(path: String) -> Result<String> {
    match eldritch::assets::Asset::get(path.as_ref()) {
        Some(f) => Ok(String::from_utf8_lossy(&f.data).to_string()),

        // {
        //     Ok(data) => data,
        //     Err(_err) => {
        //         #[cfg(debug_assertions)]
        //         log::error!("failed to load install asset: {_err}");

        //         return
        //     },
        // },
        None => {
            #[cfg(debug_assertions)]
            log::error!("no asset file at {}", path);

            Err(anyhow!("no asset file at {}", path))
        }
    }
}
