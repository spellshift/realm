// Wait, xterm.js has an `onData` handler. If we paste text, it usually fires `onData` with the pasted text.
// If the pasted text contains multiple lines, it's one large string.
// Our current handler only looks at the FIRST character (`code = data.charCodeAt(0)`) and makes a decision!
// If first character is 'h', it assumes the entire string is just text, and appends it.
// If first character is '\r', it assumes the entire string is an Enter press, and completely ignores the rest!
// We need to parse the incoming `data` string.

// BUT wait, can we just use xterm's native copy-paste, or are we manually doing it?
// The problem is that our handler processes `data` as ONE keypress.
// This is fundamentally broken for copy-paste because a pasted block arrives as a single `data` payload.
// If we paste "hello\nworld", `code` is 'h'. It goes to `code >= 32` and appends "hello\nworld" to `state.inputBuffer`.
// Then when the user presses Enter, `adapter.current?.input("hello\nworld")` happens.
// Which might actually work if the adapter supports multi-line strings, but wait...
// If we paste "hello", it becomes "hello", which works.
// So why does "Copy + Paste does not work"?
// Maybe `Ctrl+V` (Code 22) or `Ctrl+C` (Code 3) is intercepted?
// In Windows/Linux, `Ctrl+C` sends `\x03` (SIGINT). In xterm.js, if you press Ctrl+C, `onData` receives `\x03`.
// If you press `Ctrl+V`, `onData` receives `\x16`!
// And xterm.js doesn't natively map Ctrl+V to paste! You have to handle `paste` event!
// Oh!
