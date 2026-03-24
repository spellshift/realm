use eldritch::Interpreter;
use eldritch_agent::Agent;
use eldritch_libchain::std::StdChainLibrary;
use std::sync::Arc;

/// Register all pro Eldritch libraries (load, chain) onto the given interpreter.
/// Call this instead of individual `register_lib` invocations so that pro library
/// registration is maintained in a single place.
pub fn register_pro_interpreter(interpreter: &mut Interpreter, agent: Arc<dyn Agent>) {
    interpreter.register_lib(StdChainLibrary::new(agent));
}
