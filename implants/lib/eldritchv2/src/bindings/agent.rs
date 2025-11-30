use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;

#[eldritch_library("agent")]
pub trait AgentLibrary {
    #[eldritch_method]
    fn eval(&self, script: String) -> Result<(), String>;

    #[eldritch_method]
    fn set_callback_interval(&self, new_interval: i64) -> Result<(), String>;

    #[eldritch_method]
    fn set_callback_uri(&self, new_uri: String) -> Result<(), String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(AgentLibrary)]
pub struct AgentLibraryFake;

#[cfg(feature = "fake_bindings")]
impl AgentLibrary for AgentLibraryFake {
    fn eval(&self, _script: String) -> Result<(), String> {
        Ok(())
    }

    fn set_callback_interval(&self, _new_interval: i64) -> Result<(), String> {
        Ok(())
    }

    fn set_callback_uri(&self, _new_uri: String) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_agent_fake() {
        let agent = AgentLibraryFake::default();
        agent.eval("print('hello')".into()).unwrap();
        agent.set_callback_interval(10).unwrap();
        agent.set_callback_uri("http://localhost".into()).unwrap();
    }
}
