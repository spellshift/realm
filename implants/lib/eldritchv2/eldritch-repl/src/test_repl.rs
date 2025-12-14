#[cfg(test)]
extern crate alloc;

mod tests {
    use crate::{Input, Repl, ReplAction};
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_clear_screen() {
        let mut repl = Repl::new();
        let action = repl.handle_input(Input::ClearScreen);
        assert_eq!(action, ReplAction::ClearScreen);
    }

    #[test]
    fn test_history_search() {
        let mut repl = Repl::new();
        repl.load_history(vec![
            "print(1)".to_string(),
            "x = 2".to_string(),
            "print(3)".to_string(),
        ]);

        // Start search
        let action = repl.handle_input(Input::HistorySearch);
        assert_eq!(action, ReplAction::Render);
        let state = repl.get_render_state();
        assert!(state.prompt.contains("reverse-i-search"));

        // Type 'print'
        repl.handle_input(Input::Char('p'));
        repl.handle_input(Input::Char('r'));
        repl.handle_input(Input::Char('i'));
        repl.handle_input(Input::Char('n'));
        repl.handle_input(Input::Char('t'));

        let state = repl.get_render_state();
        // Should find the most recent "print(3)"
        assert_eq!(state.buffer, "print(3)");

        // Search again (Ctrl+R)
        repl.handle_input(Input::HistorySearch);
        let state = repl.get_render_state();
        // Should find "print(1)"
        assert_eq!(state.buffer, "print(1)");

        // Search again (fail/stay)
        repl.handle_input(Input::HistorySearch);
        let state = repl.get_render_state();
        // Should stay at "print(1)" or loop?
        // My impl loops if no more match found? Or stops.
        // Logic: search backwards from current match_idx.
        // If match_idx is 0, loop doesn't run.
        assert_eq!(state.buffer, "print(1)");

        // Accept (Enter)
        let action = repl.handle_input(Input::Enter);
        assert_eq!(action, ReplAction::Render); // Just ends search, returns to normal mode with buffer set
        let state = repl.get_render_state();
        assert_eq!(state.prompt, ">>> ");
        assert_eq!(state.buffer, "print(1)");
    }
}
