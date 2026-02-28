// The problem: if `data` is longer than 1 char, and it contains newlines, our code sees `code >= 32` (assuming the first character isn't a control char), and simply dumps `data` into `state.inputBuffer`.
// E.g. data = "abc\ndef"
// state.inputBuffer becomes "abc\ndef"
// then later user presses Enter, but xterm might not even show it well, or it doesn't process the commands correctly.

// If `data` is just "abc" (no newlines), it goes into the `else` branch of `code >= 32 && code !== 127`:
// state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
// This actually works perfectly for copying & pasting a single line.
// Wait, why does the issue say "Copy + Paste does not work"?
// If they paste a single line "abc", it appends "abc" at cursor. It should work?
// Let's check `code >= 32` again. Wait, if we paste using Ctrl+V or Cmd+V, it usually triggers `onData`.
// Let's create a quick xterm test to see if pasting triggers something else, or what exactly `data` contains.
