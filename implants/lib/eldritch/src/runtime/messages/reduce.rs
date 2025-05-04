use super::{report_agg_output::ReportAggOutputMessage, AsyncMessage, Message};
use pb::c2::TaskError;

pub(crate) fn reduce(mut messages: Vec<Message>) -> Vec<Message> {
    let mut id = 0;
    let mut exec_finished_at = None;
    let mut exec_started_at = None;
    let mut error = String::new();
    let mut text = String::new();

    let mut idx = 0;
    while idx < messages.len() {
        match &mut messages[idx] {
            Message::Async(am) => match am {
                AsyncMessage::ReportStart(msg) => {
                    #[cfg(debug_assertions)]
                    if id != 0 && msg.id != id {
                        log::warn!("overwriting conflicting id (old={},new={})", id, msg.id);
                    }

                    id = msg.id;
                    if exec_started_at.is_none() {
                        exec_started_at = Some(msg.exec_started_at.clone());
                    }
                    messages.remove(idx);
                }
                AsyncMessage::ReportFinish(msg) => {
                    #[cfg(debug_assertions)]
                    if id != 0 && msg.id != id {
                        log::warn!("overwriting conflicting id (old={},new={})", id, msg.id);
                    }

                    id = msg.id;
                    if exec_finished_at.is_none() {
                        exec_finished_at = Some(msg.exec_finished_at.clone());
                    }
                    messages.remove(idx);
                }
                AsyncMessage::ReportText(msg) => {
                    #[cfg(debug_assertions)]
                    if id != 0 && msg.id != id {
                        log::warn!("overwriting conflicting id (old={},new={})", id, msg.id);
                    }

                    id = msg.id;
                    text.push_str(&msg.text);
                    messages.remove(idx);
                }
                AsyncMessage::ReportError(msg) => {
                    #[cfg(debug_assertions)]
                    if id != 0 && msg.id != id {
                        log::warn!("overwriting conflicting id (old={},new={})", id, msg.id);
                    }

                    id = msg.id;
                    error.push_str(&msg.error);
                    messages.remove(idx);
                }
                _ => {
                    idx += 1;
                }
            },
            _ => {
                idx += 1;
            }
        };
    }

    // Add Aggregated Message (if available)
    if id != 0 {
        messages.push(
            AsyncMessage::from(ReportAggOutputMessage {
                id,
                text,
                error: if error.is_empty() {
                    None
                } else {
                    Some(TaskError { msg: error })
                },
                exec_started_at,
                exec_finished_at,
            })
            .into(),
        );
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::{Message, ReportAggOutputMessage};
    use crate::runtime::messages::*;
    use pb::c2::*;
    use pb::eldritch::credential;
    use pb::eldritch::process;
    use pb::eldritch::*;
    use prost_types::Timestamp;

    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let tc = $value;
                let messages = super::reduce(tc.messages);
                assert_eq!(tc.want_messages, messages);
            }
        )*
        }
    }

    struct TestCase {
        messages: Vec<Message>,
        want_messages: Vec<Message>,
    }

    test_cases!(
        empty: TestCase {
            messages: Vec::new(),
            want_messages: Vec::new(),
        },
        multi_text: TestCase {
            messages: vec![
                AsyncMessage::from(ReportTextMessage{
                    id: 12345,
                    text: String::from("abc"),
                }).into(),
                AsyncMessage::from(ReportTextMessage{
                    id: 12345,
                    text: String::from("defg"),
                }).into(),
            ],
            want_messages: vec![
                AsyncMessage::from(ReportAggOutputMessage{
                    id: 12345,
                    text: String::from("abcdefg"),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }).into(),
            ],
        },
        multi_err: TestCase {
            messages: vec![
                AsyncMessage::from(ReportErrorMessage{
                    id: 12345,
                    error: String::from("abc"),
                }).into(),
                AsyncMessage::from(ReportErrorMessage{
                    id: 12345,
                    error: String::from("defg"),
                }).into(),
            ],
            want_messages: vec![
                AsyncMessage::from(ReportAggOutputMessage{
                    id: 12345,
                    error: Some(TaskError{
                        msg: String::from("abcdefg"),
                    }),
                    text: String::new(),
                    exec_started_at: None,
                    exec_finished_at: None,
                }).into(),
            ],
        },
        complex: TestCase {
            messages: vec![
                AsyncMessage::from(ReportStartMessage{
                    id: 12345,
                    exec_started_at: Timestamp{
                        seconds: 998877,
                        nanos: 1337,
                    },
                }).into(),
                AsyncMessage::from(ReportProcessListMessage{
                    id: 123456,
                    list: ProcessList{list: vec![
                        Process{
                            pid: 5,
                            ppid: 101,
                            name: "test".to_string(),
                            principal: "root".to_string(),
                            path: "/bin/cat".to_string(),
                            env: "COOL=1".to_string(),
                            cmd: "cat".to_string(),
                            cwd: "/home/meow".to_string(),
                            status: process::Status::Idle.into(),
                        },
                        Process{
                            pid: 5,
                            ppid: 101,
                            name: "test".to_string(),
                            principal: "root".to_string(),
                            path: "/bin/cat".to_string(),
                            env: "COOL=1".to_string(),
                            cmd: "cat".to_string(),
                            cwd: "/home/meow".to_string(),
                            status: process::Status::Idle.into(),
                        },
                    ]},
                }).into(),
                AsyncMessage::from(ReportTextMessage{
                    id: 12345,
                    text: String::from("meow"),
                }).into(),
                AsyncMessage::from(ReportCredentialMessage{
                    id: 5678,
                    credential: Credential{
                        principal: String::from("roboto"),
                        secret: String::from("domo arigato mr."),
                        kind: credential::Kind::Password.into(),
                    }
                }).into(),
                AsyncMessage::from(ReportErrorMessage{
                    id: 12345,
                    error: String::from("part of an "),
                }).into(),
                AsyncMessage::from(ReportCredentialMessage{
                    id: 9876,
                    credential: Credential{
                        principal: String::from("roboto"),
                        secret: String::from("domo arigato mr."),
                        kind: credential::Kind::Password.into(),
                    }
                }).into(),
                AsyncMessage::from(ReportTextMessage{
                    id: 12345,
                    text: String::from(";bark"),
                }).into(),
                AsyncMessage::from(ReportErrorMessage{
                    id: 12345,
                    error: String::from("error.\n done."),
                }).into(),
                AsyncMessage::from(ReportFinishMessage{
                    id: 12345,
                    exec_finished_at: Timestamp{
                        seconds: 998877666,
                        nanos: 4201337,
                    },
                }).into(),
            ],
            want_messages: vec![
                AsyncMessage::from(ReportProcessListMessage{
                    id: 123456,
                    list: ProcessList{list: vec![
                        Process{
                            pid: 5,
                            ppid: 101,
                            name: "test".to_string(),
                            principal: "root".to_string(),
                            path: "/bin/cat".to_string(),
                            env: "COOL=1".to_string(),
                            cmd: "cat".to_string(),
                            cwd: "/home/meow".to_string(),
                            status: process::Status::Idle.into(),
                        },
                        Process{
                            pid: 5,
                            ppid: 101,
                            name: "test".to_string(),
                            principal: "root".to_string(),
                            path: "/bin/cat".to_string(),
                            env: "COOL=1".to_string(),
                            cmd: "cat".to_string(),
                            cwd: "/home/meow".to_string(),
                            status: process::Status::Idle.into(),
                        },
                    ]},
                }).into(),
                AsyncMessage::from(ReportCredentialMessage{
                    id: 5678,
                    credential: Credential{
                        principal: String::from("roboto"),
                        secret: String::from("domo arigato mr."),
                        kind: credential::Kind::Password.into(),
                    }
                }).into(),
                AsyncMessage::from(ReportCredentialMessage{
                    id: 9876,
                    credential: Credential{
                        principal: String::from("roboto"),
                        secret: String::from("domo arigato mr."),
                        kind: credential::Kind::Password.into(),
                    }
                }).into(),
                AsyncMessage::from(ReportAggOutputMessage{
                    id: 12345,
                    error: Some(TaskError{
                        msg: String::from("part of an error.\n done."),
                    }),
                    text: String::from("meow;bark"),
                    exec_started_at: Some(Timestamp{
                        seconds: 998877,
                        nanos: 1337,
                    }),
                    exec_finished_at: Some(Timestamp{
                        seconds: 998877666,
                        nanos: 4201337,
                    }),
                }).into(),
            ],
        },
    );
}
