use crate::PortalLibrary;
use alloc::string::{String, ToString};
use eldritch_agent::{Agent, Context};
use eldritch_macros::eldritch_library_impl;
use std::sync::Arc;

#[derive(Default)]
#[eldritch_library_impl(PortalLibrary)]
pub struct StdPortalLibrary {
    pub agent: Option<Arc<dyn Agent>>,
    pub context: Option<Context>,
}

impl core::fmt::Debug for StdPortalLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdPortalLibrary")
            .field("context", &self.context)
            .finish()
    }
}

impl StdPortalLibrary {
    pub fn new(agent: Arc<dyn Agent>, context: Context) -> Self {
        Self {
            agent: Some(agent),
            context: Some(context),
        }
    }
}

impl PortalLibrary for StdPortalLibrary {
    fn create(&self) -> Result<(), String> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| "No agent available".to_string())?;
        let context = self
            .context
            .clone()
            .ok_or_else(|| "No context available".to_string())?;
        agent.create_portal(context).map_err(|e| e.to_string())
    }
}
