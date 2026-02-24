use uuid::Uuid;

mod env;
pub use env::Env;
mod file;
pub use file::File;
mod macaddr;
pub use macaddr::MacAddr;
mod registry;
pub use registry::Registry;

pub trait HostIDSelector {
    fn get_name(&self) -> String;
    fn get_host_id(&self) -> Option<Uuid>;
}

/// Describes a single uniqueness selector as encoded in `IMIX_UNIQUE`.
///
/// The JSON representation uses `type` as a tag and `args` as the
/// selector struct directly (its fields become the args object keys).
/// Example:
///
/// ```json
/// [
///   {"type": "env"},
///   {"type": "macaddr"},
///   {"type": "file",        "args": {"path_override": "/custom/path"}},
///   {"type": "registry",    "args": {"subkey": "SOFTWARE\\Custom"}}
/// ]
/// ```
///
/// Adding a new selector requires only:
/// 1. Adding `#[derive(Serialize, Deserialize)]` to the selector struct.
/// 2. Adding a variant here — serde handles JSON marshaling automatically.
#[derive(serde::Serialize)]
#[serde(tag = "type", content = "args")]
pub enum SelectorSpec {
    #[serde(rename = "env")]
    Env(Env),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "macaddr")]
    MacAddr(MacAddr),
    #[serde(rename = "registry")]
    Registry(Registry),
}

/// Helper type identical to [`SelectorSpec`] used only to derive the
/// adjacently-tagged deserializer.  [`SelectorSpec`] itself has a custom
/// [`Deserialize`] impl that inserts a default `"args": {}` when the key
/// is absent, then delegates to this derived impl.
#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "args")]
enum SelectorSpecDerived {
    #[serde(rename = "env")]
    Env(Env),
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "macaddr")]
    MacAddr(MacAddr),
    #[serde(rename = "registry")]
    Registry(Registry),
}

impl From<SelectorSpecDerived> for SelectorSpec {
    fn from(d: SelectorSpecDerived) -> Self {
        match d {
            SelectorSpecDerived::Env(v) => SelectorSpec::Env(v),
            SelectorSpecDerived::File(v) => SelectorSpec::File(v),
            SelectorSpecDerived::MacAddr(v) => SelectorSpec::MacAddr(v),
            SelectorSpecDerived::Registry(v) => SelectorSpec::Registry(v),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SelectorSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;

        let mut v = Value::deserialize(deserializer)?;
        // Insert a default empty "args" when the key is absent so that the
        // derived adjacently-tagged deserializer always finds the content field.
        if let Some(obj) = v.as_object_mut() {
            obj.entry("args")
                .or_insert_with(|| Value::Object(Default::default()));
        }
        SelectorSpecDerived::deserialize(v)
            .map(SelectorSpec::from)
            .map_err(serde::de::Error::custom)
    }
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
pub fn from_imix_unique(json: String) -> Option<Vec<Box<dyn HostIDSelector>>> {
    let specs: Vec<SelectorSpec> = serde_json::from_str(&json)
        .expect("IMIX_UNIQUE contains invalid JSON - this should have been caught at build time");
    Some(specs.into_iter().map(SelectorSpec::into_selector).collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_spec_roundtrip() {
        let json = r#"[
            {"type": "env",         "args": {}},
            {"type": "macaddr", "args": {}},
            {"type": "file",        "args": {"path_override": "/tmp/test-id"}},
            {"type": "registry",    "args": {"subkey": "SOFTWARE\\Test", "value_name": null}}
        ]"#;

        let specs: Vec<SelectorSpec> = serde_json::from_str(json).expect("parse failed");
        assert_eq!(specs.len(), 4);

        // Verify each spec produces a valid selector with the expected name.
        let names: Vec<String> = specs
            .into_iter()
            .map(|s| s.into_selector().get_name())
            .collect();
        assert_eq!(names, ["env", "macaddr", "file", "registry"]);
    }

    #[test]
    fn test_selector_spec_optional_args() {
        let json = r#"[
            {"type": "env"},
            {"type": "macaddr"}
        ]"#;

        let specs: Vec<SelectorSpec> = serde_json::from_str(json).expect("parse failed");
        assert_eq!(specs.len(), 2);

        let names: Vec<String> = specs
            .into_iter()
            .map(|s| s.into_selector().get_name())
            .collect();
        assert_eq!(names, ["env", "macaddr"]);
    }

    #[test]
    fn test_selector_spec_serialize() {
        let specs = vec![
            SelectorSpec::Env(Env::default()),
            SelectorSpec::MacAddr(MacAddr::default()),
        ];
        let json = serde_json::to_string(&specs).expect("serialize failed");
        assert!(json.contains(r#""type":"env""#));
        assert!(json.contains(r#""type":"macaddr""#));
        assert!(json.contains(r#""args":{}"#));
    }
}
