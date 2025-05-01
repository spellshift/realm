use std::collections::HashMap;

use crate::runtime::{Environment, Runtime};
use anyhow::Result;
use pb::eldritch::Tome;

pub fn eval(env: &Environment, script: String) -> Result<()> {
    let tome = Tome {
        eldritch: script,
        parameters: HashMap::new(),
        file_names: Vec::new(),
    };
    Runtime::run(env, &tome)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use crate::runtime::Message;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_eval() -> Result<()> {
        let (tx, _rx) = channel::<Message>();
        let test_env = Environment::mock(1, tx);
        eval(&test_env, String::from("print(\"hi\")"))
    }

    #[test]
    fn test_eval_fail() -> Result<()> {
        let (tx, _rx) = channel::<Message>();
        let test_env = Environment::mock(1, tx);
        assert!(eval(&test_env, String::from("blorp(\"hi\")")).is_err());
        Ok(())
    }
}
