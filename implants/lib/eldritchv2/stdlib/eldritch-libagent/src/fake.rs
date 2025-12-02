
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;
use super::AgentLibrary;

#[derive(Default, Debug)]
#[eldritch_library_impl(AgentLibrary)]
pub struct AgentLibraryFake;

impl AgentLibrary for AgentLibraryFake {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_id(&self) -> Result<String, String> {
        Ok(String::from("fake-agent-uuid"))
    }

    fn get_platform(&self) -> Result<String, String> {
        Ok(String::from("linux"))
    }

    fn kill(&self) -> Result<(), String> {
        Ok(())
    }

    fn set_config(&self, _config: BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn sleep(&self, _secs: i64) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {


    #[test]
    fn test_agent_fake() {
        let agent = AgentLibraryFake::default();
        assert_eq!(agent.get_id().unwrap(), "fake-agent-uuid");
    }
}
