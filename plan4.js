// Let's look at `termInstance.current.attachCustomKeyEventHandler`
// It allows us to intercept key events. If e.ctrlKey && e.key === 'v', we can read from clipboard.
// Or if e.ctrlKey && e.key === 'c', we can write to clipboard.
// But wait, xterm has a built-in clipboard support.
// You just need to right-click to paste, or shift+insert, or use the browser's paste event.
// Let's see if xterm's `onData` receives the pasted text. If we right-click and paste in the browser, does `onData` receive it?
// Yes, xterm handles the browser's native `paste` event and fires `onData` with the pasted text.
// So if the issue is "Copy + Paste does not work", it's probably because:
// 1. Ctrl+C copies text? By default, Ctrl+C in xterm sends \x03 to the backend (which we handle as `adapter.current?.reset()`), instead of copying the selected text.
// 2. Ctrl+V pastes text? By default, xterm sends \x16 to the backend, instead of pasting.

// To fix copy and paste in xterm.js:
// 1. We should configure xterm to handle Ctrl+C as copy if there is a selection.
// 2. We should configure xterm to handle Ctrl+V as paste.
// 3. AND we need to process pasted data correctly if it contains newlines.
