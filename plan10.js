// So the user says: "Copy + Paste does not work in our shellv2, please ensure this has a smooth user experience"

// 1. Copy: Usually in xterm, if you highlight text, you want Ctrl+C to copy if something is highlighted, instead of sending SIGINT (`\x03`) to the backend.
// Currently, `if (data === "\x03") { // Ctrl+C` it always sends reset.
// We should check `term.hasSelection()`! If true, we should `document.execCommand('copy')` or `navigator.clipboard.writeText(term.getSelection())`, and `term.clearSelection()`, instead of resetting the prompt.
// Oh wait, `onData` receives `\x03` when we press Ctrl+C.
// We can change:
// `if (data === "\x03") { // Ctrl+C`
// `    if (term.hasSelection()) {`
// `        navigator.clipboard.writeText(term.getSelection());`
// `        term.clearSelection();`
// `        return;`
// `    }`
// `    adapter.current?.reset(); ... `
// That fixes Copy!

// 2. Paste:
// The user might press Ctrl+V, which xterm interprets as `\x16`!
// Wait, in a browser, Ctrl+V on Windows triggers a native paste IF the textarea has focus.
// But sometimes it doesn't, or people want Shift+Insert or right-click.
// Actually, `xterm` has `attachCustomKeyEventHandler` to intercept keys BEFORE they generate data sequences.
// Let's implement that!
