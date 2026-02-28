// WAIT, xterm's native copy DOES NOT intercept `\x03`!
// If you press Ctrl+C, xterm generates an `onData('\x03')` event.
// BUT xterm also has a `hasSelection()` method.
// So in `useShellTerminal.ts`:
// ```typescript
// if (data === "\x03") { // Ctrl+C
//    if (term.hasSelection()) {
//        navigator.clipboard.writeText(term.getSelection());
//        term.clearSelection();
//        return; // do not send SIGINT to backend
//    }
//    adapter.current?.reset();
//    ...
// ```
// This fixes COPY!

// Now for PASTE!
// How do we detect a paste?
// If `data` contains multiple characters AND it is not an escape sequence (like `\x1b[A`), it's likely a paste.
// But wait, xterm handles `paste` event by calling `onData(pastedText)`.
// `pastedText` can be anything!
// If `data` is longer than 1 character, it could be an ANSI sequence (like Arrow Up).
// ANSI sequences always start with `\x1b`.
// If `data.length > 1 && !data.startsWith("\x1b")`, then it's definitely pasted text!
// Or we can just loop over all characters! BUT wait, if we loop over `\x1b[A`, it will process `\x1b`, `[`, `A` separately, breaking arrow keys!
// Ah! This is why `data.charCodeAt(0)` is used to decide the branch!
// For Arrow Up (`\x1b[A`), `code` is 27 (`\x1b`).
// Our code handles `data === "\x1b[A"`.
