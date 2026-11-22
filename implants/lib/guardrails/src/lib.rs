mod file;
pub use file::File;
mod process;
pub use process::Process;
mod registry;
pub use registry::Registry;

pub trait Guardrail {
    fn get_name(&self) -> String;
    fn check(&self) -> bool;
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum GuardrailSpec {
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "process")]
    Process(Process),
    #[serde(rename = "registry")]
    Registry(Registry),
}

impl GuardrailSpec {
    pub fn into_guardrail(self) -> Box<dyn Guardrail> {
        match self {
            GuardrailSpec::File(g) => Box::new(g),
            GuardrailSpec::Process(g) => Box::new(g),
            GuardrailSpec::Registry(g) => Box::new(g),
        }
    }
}

/// Evaluate configured guardrails using OR semantics:
/// - Empty list: returns `true` (no guardrails configured).
/// - Non-empty list: returns `true` if at least one guardrail passes, otherwise `false`.
pub fn check_guardrails(guardrails: Vec<Box<dyn Guardrail>>) -> bool {
    if guardrails.is_empty() {
        return true;
    }

    for guardrail in guardrails {
        if guardrail.check() {
            #[cfg(feature = "print_debug")]
            log::debug!("Guardrail {} passed", guardrail.get_name());
            return true;
        }
        #[cfg(feature = "print_debug")]
        log::debug!("Guardrail {} failed", guardrail.get_name());
    }
    false
}

/// Parse guardrail JSON specifications without panicking.
/// Returns `None` on malformed JSON, unknown guardrail types, or invalid args
/// so callers can fail closed.
pub fn from_imix_guardrails(json: String) -> Option<Vec<Box<dyn Guardrail>>> {
    let specs: Vec<GuardrailSpec> = serde_json::from_str(&json).ok()?;
    Some(
        specs
            .into_iter()
            .map(GuardrailSpec::into_guardrail)
            .collect(),
    )
}

pub fn defaults() -> Vec<Box<dyn Guardrail>> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StaticGuardrail {
        name: &'static str,
        result: bool,
    }

    impl Guardrail for StaticGuardrail {
        fn get_name(&self) -> String {
            self.name.to_string()
        }

        fn check(&self) -> bool {
            self.result
        }
    }

    #[test]
    fn test_guardrail_spec_roundtrip() {
        let json = r#"[
            {"type": "file", "args": {"path": "/tmp/test-id"}},
            {"type": "process", "args": {"name": "explorer.exe"}},
            {"type": "registry", "args": {"path": "HKLM\\SOFTWARE\\Test", "value_name": null}}
        ]"#;

        let specs: Vec<GuardrailSpec> = serde_json::from_str(json).expect("parse failed");
        assert_eq!(specs.len(), 3);

        let names: Vec<String> = specs
            .into_iter()
            .map(|s| s.into_guardrail().get_name())
            .collect();
        assert_eq!(names, ["file", "process", "registry"]);
    }

    #[test]
    fn test_from_imix_guardrails_rejects_malformed_without_panic() {
        assert!(from_imix_guardrails("not json".to_string()).is_none());
        assert!(from_imix_guardrails("{}".to_string()).is_none());
        assert!(from_imix_guardrails(r#"[{"type": "unknown", "args": {}}]"#.to_string()).is_none());
        assert!(from_imix_guardrails(r#"[{"type": "file"}]"#.to_string()).is_none());
        assert!(from_imix_guardrails(
            r#"[{"type": "file", "args": {"wrong": "/tmp/a"}}]"#.to_string()
        )
        .is_none());
        assert!(from_imix_guardrails(
            r#"[{"type": "process", "args": {"name": "sshd", "extra": 1}}]"#.to_string()
        )
        .is_none());
    }

    #[test]
    fn test_check_guardrails_or_semantics() {
        assert!(check_guardrails(vec![]));

        let one_passes: Vec<Box<dyn Guardrail>> = vec![
            Box::new(StaticGuardrail {
                name: "fail",
                result: false,
            }),
            Box::new(StaticGuardrail {
                name: "pass",
                result: true,
            }),
        ];
        assert!(check_guardrails(one_passes));

        let all_fail: Vec<Box<dyn Guardrail>> = vec![
            Box::new(StaticGuardrail {
                name: "fail1",
                result: false,
            }),
            Box::new(StaticGuardrail {
                name: "fail2",
                result: false,
            }),
        ];
        assert!(!check_guardrails(all_fail));
    }
}
