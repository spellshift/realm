use debugserver_types::*;
use gazebo::prelude::*;
use serde::Serialize;
use serde_json::Value;

use crate::dap::library::stream::{log, send};

#[derive(Debug, Clone, Dupe)]
pub struct Client {
    _private: (),
}

impl Client {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }

    pub fn log(&self, x: &str) {
        log(x)
    }

    fn event(&self, x: impl Serialize) {
        send(serde_json::to_value(&x).unwrap())
    }

    pub fn event_stopped(&self, body: StoppedEventBody) {
        self.event(StoppedEvent {
            type_: "event".to_owned(),
            seq: 0,
            event: "stopped".to_owned(),
            body,
        })
    }

    pub fn event_initialized(&self, body: Option<Value>) {
        self.event(InitializedEvent {
            type_: "event".to_owned(),
            seq: 0,
            event: "initialized".to_owned(),
            body,
        })
    }

    pub fn event_exited(&self, body: ExitedEventBody) {
        self.event(ExitedEvent {
            type_: "event".to_owned(),
            seq: 0,
            event: "exited".to_owned(),
            body,
        })
    }

    pub fn event_terminated(&self, body: Option<TerminatedEventBody>) {
        self.event(TerminatedEvent {
            type_: "event".to_owned(),
            seq: 0,
            event: "terminated".to_owned(),
            body,
        })
    }

    pub fn event_output(&self, body: OutputEventBody) {
        self.event(OutputEvent {
            type_: "event".to_owned(),
            seq: 0,
            event: "output".to_owned(),
            body,
        })
    }
}