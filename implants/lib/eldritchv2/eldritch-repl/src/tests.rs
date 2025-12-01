use crate::lang::ast::{Environment, Value};
#[cfg(test)]
use crate::repl::{Input, Repl, ReplAction};
use crate::Interpreter;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use core::cell::RefCell;

#[test]
fn test_repl_basic_input() {
    let mut repl = Repl::new();
    let action = repl.handle_input(Input::Char('a'));
    assert_eq!(action, ReplAction::Render);
    let state = repl.get_render_state();
    assert_eq!(state.buffer, "a");
    assert_eq!(state.cursor, 1);
}

#[test]
fn test_repl_movement() {
    let mut repl = Repl::new();
    repl.handle_input(Input::Char('a'));
    repl.handle_input(Input::Char('b'));

    // Move left
    repl.handle_input(Input::Left);
    let state = repl.get_render_state();
    assert_eq!(state.cursor, 1);

    // Insert 'c' at cursor 1 -> "acb"
    repl.handle_input(Input::Char('c'));
    let state = repl.get_render_state();
    assert_eq!(state.buffer, "acb");
    assert_eq!(state.cursor, 2);

    // Move home
    repl.handle_input(Input::Home);
    assert_eq!(repl.get_render_state().cursor, 0);

    // Move end
    repl.handle_input(Input::End);
    assert_eq!(repl.get_render_state().cursor, 3);
}

#[test]
fn test_repl_history() {
    let mut repl = Repl::new();
    repl.load_history(vec!["h1".to_string(), "h2".to_string()]);

    // Up -> h2
    repl.handle_input(Input::Up);
    assert_eq!(repl.get_render_state().buffer, "h2");

    // Up -> h1
    repl.handle_input(Input::Up);
    assert_eq!(repl.get_render_state().buffer, "h1");

    // Down -> h2
    repl.handle_input(Input::Down);
    assert_eq!(repl.get_render_state().buffer, "h2");

    // Down -> empty/original
    repl.handle_input(Input::Down);
    assert_eq!(repl.get_render_state().buffer, "");
}

#[test]
fn test_repl_multiline() {
    let mut repl = Repl::new();

    // Type partial block: "if true {"
    let input = "if true {";
    for c in input.chars() {
        repl.handle_input(Input::Char(c));
    }

    // Enter -> Should continue (AcceptLine)
    let action = repl.handle_input(Input::Enter);
    match action {
        ReplAction::AcceptLine { line, prompt } => {
            assert_eq!(line, "if true {");
            // The prompt used for this line was the default one
            assert!(prompt.contains(">>>"));
        }
        _ => panic!("Expected AcceptLine, got {:?}", action),
    }

    // Verify next prompt is continuation
    assert!(repl.get_render_state().prompt.contains("..."));

    // Type "}"
    repl.handle_input(Input::Char('}'));

    // Enter -> Should AcceptLine (because multiline block requires empty line to submit by default logic)
    let action = repl.handle_input(Input::Enter);
    match action {
        ReplAction::AcceptLine { line, prompt } => {
            assert_eq!(line, "}");
            assert!(prompt.contains("..."));
        }
        _ => panic!(
            "Expected AcceptLine (waiting for empty line), got {:?}",
            action
        ),
    }

    // Enter (empty line) -> Should Submit
    let action = repl.handle_input(Input::Enter);
    match action {
        ReplAction::Submit { code, .. } => {
            // Full code includes newlines
            // "if true {\n}\n" or similar.
            // Let's just check it contains the parts.
            assert!(code.contains("if true {"));
            assert!(code.contains("}"));
        }
        _ => panic!("Expected Submit, got {:?}", action),
    }
}

// Helper to match BuiltinFn signature
fn mock_function(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Bool(true));
    }
    Ok(args[0].clone())
}

#[test]
fn test_register_function() {
    let mut interp = Interpreter::new();

    // Register the mock function
    interp.register_function("mock_fn", mock_function);

    // Call it from Eldritch
    let result = interp.interpret("mock_fn(123)");
    assert_eq!(result, Ok(Value::Int(123)));

    let result_empty = interp.interpret("mock_fn()");
    assert_eq!(result_empty, Ok(Value::Bool(true)));
}
