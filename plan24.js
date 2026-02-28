// WAIT, what if the pasted text has ANSI sequences? e.g. `\x1b[A`
// If I iterate character by character:
// `char = '\x1b'`
// `char = '['`
// `char = 'A'`
// The original code checks `if (data === "\x1b[A")`!
// If I break it into characters, `handleChar("\x1b")` will receive ESC (code 27). It cancels completions.
// `handleChar("[")` will write `[`.
// `handleChar("A")` will write `A`.
// This breaks escape sequences!

// So I should only split the pasted string if it doesn't contain an escape sequence.
// Actually, xterm handles pasted text by sending the entire text in one `onData` event.
// Native key presses (like Up arrow) send exactly `\x1b[A` in one `onData` event.
// So:
// ```typescript
// if (data.length > 1 && !data.includes("\x1b")) {
//    // This is a paste or macro string, process character by character
//    // Wait, what if we just split by \r or \n and process lines?
// }
// ```
