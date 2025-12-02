#[cfg(test)]
mod tests {
    use crate::runtime::{messages::AsyncMessage, Message};
    use pb::eldritch::Tome;
    use std::collections::HashMap;

    macro_rules! parity_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                let mut runtime = crate::start(tc.id, tc.tome).await;
                runtime.finish().await;

                let mut text = Vec::new();
                for msg in runtime.messages() {
                    match msg {
                        Message::Async(am) => {
                            match am {
                                AsyncMessage::ReportText(m) => text.push(m.text),
                                AsyncMessage::ReportError(m) => assert_eq!(tc.want_error, Some(m.error)),
                                _ => {},
                            }
                        },
                        _ => {},
                    };
                }

                // If we expect output, check it. If we expect no error, we already asserted that.
                if !tc.want_text.is_empty() {
                     assert_eq!(tc.want_text, text.join(""));
                }
            }
        )*
        }
    }

    struct TestCase {
        pub id: i64,
        pub tome: Tome,
        pub want_text: String,
        pub want_error: Option<String>,
    }

    fn make_tome(code: &str) -> Tome {
        Tome {
            eldritch: code.to_string(),
            parameters: HashMap::new(),
            file_names: Vec::new(),
        }
    }

    parity_tests! {
        test_zip: TestCase {
            id: 3,
            tome: make_tome("print(zip([1, 2], ['a', 'b']))"),
            want_text: "[(1, \"a\"), (2, \"b\")]\n".to_string(),
            want_error: None,
        },
        test_enumerate: TestCase {
            id: 4,
            tome: make_tome("print(enumerate(['a', 'b']))"),
            want_text: "[(0, \"a\"), (1, \"b\")]\n".to_string(),
            want_error: None,
        },
        test_any_all: TestCase {
            id: 5,
            tome: make_tome("print(any([False, True])); print(all([True, True])); print(all([True, False]))"),
            want_text: "True\nTrue\nFalse\n".to_string(),
            want_error: None,
        },
        test_reversed: TestCase {
            id: 6,
            tome: make_tome("print(reversed([1, 2, 3]))"),
            want_text: "[3, 2, 1]\n".to_string(),
            want_error: None,
        },
        test_chr_ord: TestCase {
            id: 7,
            tome: make_tome("print(chr(65)); print(ord('A'))"),
            want_text: "A\n65\n".to_string(),
            want_error: None,
        },
        test_map: TestCase {
            id: 9,
            tome: make_tome("print(map(str, [1, 2]))"),
            want_text: "[\"1\", \"2\"]\n".to_string(),
            want_error: None,
        },
        test_partial: TestCase {
            id: 10,
            tome: make_tome("def add(x, y): return x + y\np = partial(add, 1)\nprint(p(2))"),
            want_text: "3\n".to_string(),
            want_error: None,
        },
    }
}
