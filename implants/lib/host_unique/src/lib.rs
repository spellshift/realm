use uuid::Uuid;

mod env;
pub use env::Env;
mod file;
pub use file::File;
mod mac_addr;
pub use mac_addr::MacAddr;
mod registry;
pub use registry::Registry;

pub trait HostIDSelector {
    fn get_name(&self) -> String;
    fn get_host_id(&self) -> Option<Uuid>;
}

/// Describes a single uniqueness selector as encoded in `IMIX_UNIQUE`.
///
/// The JSON representation uses `type` as a tag and `arg` as the
/// selector struct directly (its fields become the arg object keys).
/// Example:
///
/// ```json
/// [
///   {"type": "env",         "arg": {}},
///   {"type": "mac_address", "arg": {}},
///   {"type": "file",        "arg": {"path_override": "/custom/path"}},
///   {"type": "registry",    "arg": {"subkey": "SOFTWARE\\Custom"}}
/// ]
/// ```
///
/// Adding a new selector requires only:
/// 1. Adding `#[derive(Serialize, Deserialize)]` to the selector struct.
/// 2. Adding a variant here — serde handles JSON marshaling automatically.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "arg")]
pub enum SelectorSpec {
    #[serde(rename = "env")]
    Env(Env),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "mac_address")]
    MacAddr(MacAddr),
    #[serde(rename = "registry")]
    Registry(Registry),
}

impl SelectorSpec {
    /// Convert this spec into a boxed [`HostIDSelector`].
    pub fn into_selector(self) -> Box<dyn HostIDSelector> {
        match self {
            SelectorSpec::Env(s) => Box::new(s),
            SelectorSpec::File(s) => Box::new(s),
            SelectorSpec::MacAddr(s) => Box::new(s),
            SelectorSpec::Registry(s) => Box::new(s),
        }
    }
}

// Iterate through all available uniqueness selectors in order if one
// returns a UUID that's accepted
pub fn get_id_with_selectors(selectors: Vec<Box<dyn HostIDSelector>>) -> Uuid {
    for selector in selectors {
        match selector.get_host_id() {
            Some(res) => {
                return res;
            }
            None => {
                #[cfg(debug_assertions)]
                log::debug!("Unique selector {} failed", selector.get_name());
            }
        }
    }

    // All else fails give it a unique one
    Uuid::new_v4()
}

/// Parse the `IMIX_UNIQUE` value baked in at build time and return the
/// corresponding ordered list of selectors.
///
/// Returns `None` when `IMIX_UNIQUE` was not set at build time, in which
/// case callers should fall back to [`defaults`].
pub fn from_imix_unique() -> Option<Vec<Box<dyn HostIDSelector>>> {
    let json = option_env!("IMIX_UNIQUE")?;
    let specs: Vec<SelectorSpec> = serde_json::from_str(json)
        .expect("IMIX_UNIQUE contains invalid JSON - this should have been caught at build time");
    Some(specs.into_iter().map(SelectorSpec::into_selector).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_spec_roundtrip() {
        let json = r#"[
            {"type": "env",         "arg": {}},
            {"type": "mac_address", "arg": {}},
            {"type": "file",        "arg": {"path_override": "/tmp/test-id"}},
            {"type": "registry",    "arg": {"subkey": "SOFTWARE\\Test", "value_name": null}}
        ]"#;

        let specs: Vec<SelectorSpec> = serde_json::from_str(json).expect("parse failed");
        assert_eq!(specs.len(), 4);

        // Verify each spec produces a valid selector with the expected name.
        let names: Vec<String> = specs
            .into_iter()
            .map(|s| s.into_selector().get_name())
            .collect();
        assert_eq!(names, ["env", "mac_address", "file", "registry"]);
    }

    #[test]
    fn test_selector_spec_serialize() {
        let specs = vec![
            SelectorSpec::Env(Env::default()),
            SelectorSpec::MacAddr(MacAddr::default()),
        ];
        let json = serde_json::to_string(&specs).expect("serialize failed");
        assert!(json.contains(r#""type":"env""#));
        assert!(json.contains(r#""type":"mac_address""#));
        assert!(json.contains(r#""arg":{}"#));
    }
}

// Return the default list of unique selectors to evaluate
// List is evaluated in order and will take the first successful
// result.
pub fn defaults() -> Vec<Box<dyn HostIDSelector>> {
    vec![
        Box::<Env>::default(),
        Box::<File>::default(),
        // Fallback for unix systems / legacy implementation
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        ))]
        Box::new(File::new_with_file("/etc/system-id")),
        Box::<MacAddr>::default(),
    ]
}
