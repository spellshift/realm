// Actually, if we use the browser paste event, sometimes it's intercepted, or xterm passes it to `onData`.
// Let's look at `data.charCodeAt(0)`.
// If `data` is "abc", `charCodeAt(0)` is 97 (`a`). `97 >= 32` is true. `97 !== 127` is true.
// The code executes:
// state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + ...
// redrawLine();

// WAIT! What if `data` is pasted, and it's sent to onData character by character?
// If we type, `data` is 1 char. If we paste, xterm might send the whole pasted string as a single `data` event, OR it might send it character by character.
// Wait, the real problem is that if we paste "hello", data is "hello".
// We do `code = data.charCodeAt(0)`.
// Then we do:
// state.inputBuffer = state.inputBuffer.slice(...) + data + ...
// What if it contains newlines?
// `data` might be "hello\r\nworld"
// Then `state.inputBuffer` has "hello\r\nworld".
// It won't execute "hello". It just sits in the buffer.
// Then when the user presses Enter, the entire string is passed to `adapter.current?.input()`.

// Another big issue:
// If we paste something that STARTS with a newline or space, etc.
// Wait, if it's sent character by character, then `code = data.charCodeAt(0)` is fine.
// But wait! If we paste, and `data` is multiple characters, what if the user presses `Ctrl+V`?
// In normal terminal emulators, `Ctrl+V` (code 22) is literal next character. In our xterm, browser handles `Ctrl+V` or `Cmd+V` to paste text, and xterm sends the pasted text via `onData`.
// But wait, the frontend might not even have copy-paste working if the terminal steals focus and doesn't pass the `paste` event correctly? No, xterm.js supports paste natively.
// If xterm sends the pasted string as one `onData` event, we need to iterate over characters and process them!
// Our current code:
// const code = data.charCodeAt(0);
// ...
// This assumes `data` behaves uniformly based on its FIRST character.
// If data = "\x1b[A" (Up arrow), code = 27 (ESC).
// If data = "hello\nworld", code = 104 ('h').
// It falls through to `if (code >= 32 && code !== 127)`
// and simply appends the ENTIRE `data` string to `state.inputBuffer`.
// BUT if the string contains a newline, it's just added to `inputBuffer`!
// Even worse, if `data` contains an ANSI escape sequence because of a paste, it just dumps it in.
// What if we paste something starting with a newline? `\r\nhello` -> code is 13 or 10.
// `13` (Enter) is handled by `else if (code === 13)`.
// But wait! If we paste `\rhello`, code is 13.
// It will execute `Enter` logic, ignoring "hello"!
// Because `else if (code === 13)` ignores the rest of `data` entirely!
// Ah!
// So we MUST process `data` chunk by chunk or character by character.
