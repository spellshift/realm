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

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "args")]
pub enum GuardrailSpec {
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "process")]
    Process(Process),
    #[serde(rename = "registry")]
    Registry(Registry),
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "args")]
enum GuardrailSpecDerived {
    #[serde(rename = "file")]
    File(File),
    #[serde(rename = "process")]
    Process(Process),
    #[serde(rename = "registry")]
    Registry(Registry),
}

impl From<GuardrailSpecDerived> for GuardrailSpec {
    fn from(d: GuardrailSpecDerived) -> Self {
        match d {
            GuardrailSpecDerived::File(v) => GuardrailSpec::File(v),
            GuardrailSpecDerived::Process(v) => GuardrailSpec::Process(v),
            GuardrailSpecDerived::Registry(v) => GuardrailSpec::Registry(v),
        }
    }
}

impl<'de> serde::Deserialize<'de> for GuardrailSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;

        let mut v = Value::deserialize(deserializer)?;
        if let Some(obj) = v.as_object_mut() {
            obj.entry("args")
                .or_insert_with(|| Value::Object(Default::default()));
        }
        GuardrailSpecDerived::deserialize(v)
            .map(GuardrailSpec::from)
            .map_err(serde::de::Error::custom)
    }
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

pub fn check_guardrails(guardrails: Vec<Box<dyn Guardrail>>) -> bool {
    for guardrail in guardrails {
        if !guardrail.check() {
            #[cfg(debug_assertions)]
            log::debug!("Guardrail {} failed", guardrail.get_name());
            return false;
        }
    }
    true
}

pub fn from_imix_guardrails(json: String) -> Option<Vec<Box<dyn Guardrail>>> {
    let specs: Vec<GuardrailSpec> = serde_json::from_str(&json).expect(
        "IMIX_GUARDRAILS contains invalid JSON - this should have been caught at build time",
    );
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

    #[test]
    fn test_guardrail_spec_roundtrip() {
        let json = r#"[
            {"type": "file", "args": {"path": "/tmp/test-id"}},
            {"type": "process", "args": {"name": "explorer.exe"}},
            {"type": "registry", "args": {"subkey": "SOFTWARE\\Test", "value_name": null}}
        ]"#;

        let specs: Vec<GuardrailSpec> = serde_json::from_str(json).expect("parse failed");
        assert_eq!(specs.len(), 3);

        let names: Vec<String> = specs
            .into_iter()
            .map(|s| s.into_guardrail().get_name())
            .collect();
        assert_eq!(names, ["file", "process", "registry"]);
    }
}
