// WAIT, the prompt says "Copy + Paste does not work in our shellv2, please ensure this has a smooth user experience".
// Currently if the user selects text and presses Ctrl+C, xterm sends `\x03` to `onData`.
// Our code catches `\x03` (Ctrl+C) and resets the prompt! It DOES NOT copy the text, it sends a reset command!
// This breaks copy!
// So:
// 1. In `onData`, if `data === "\x03"` AND `term.hasSelection()`, copy text, clear selection, and RETURN (do not reset).
// 2. What about paste? The user presses Ctrl+V (or Cmd+V on Mac). The browser triggers native `paste`, which xterm handles and emits `onData(clipboardText)`.
// BUT if the text has multiple lines, `onData` receives it as ONE large string, e.g., `"line1\r\nline2"`.
// If `data.length > 1` and `data` contains `\r` or `\n` or `\t`, our code breaks:
// It looks at `charCodeAt(0)` which is 'l' (108 >= 32).
// It goes to `state.inputBuffer += data`.
// Then you have `"line1\r\nline2"` in `inputBuffer`. But the terminal just renders it. You can't execute it until you press Enter, and when you do, it might send `"line1\r\nline2\n"` to the adapter.
// BUT wait, a smooth paste experience should execute each line if it ends with a newline, or at least process each character as if typed!
// If we iterate through `data` character by character, we can simulate typing it!
