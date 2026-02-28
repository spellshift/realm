// WAIT, xterm actually fires an `onPaste` event when you paste.
// BUT `onData` ALSO fires with the pasted text!
// If we process the pasted text properly, xterm natively handles the system `paste` event.
// Let's modify `onData` to process a string chunk by chunk, so multi-line pastes work properly.
// Or we can just use `onPaste`! Wait, if xterm fires `onData` anyway, we just need to fix `onData`.
// But wait! If we paste `\r\n` characters in `onData`, it hits the `code >= 32` block if the string starts with a normal character!
// Because `data.charCodeAt(0)` is checked.
// If the user pastes `"a\r\nb"`, `charCodeAt(0)` is 'a' (97). It hits `code >= 32 && code !== 127`, and the WHOLE string `"a\r\nb"` is added to `state.inputBuffer`!
// This is exactly why it's broken! It just dumps the text.

// But wait, what if we press Ctrl+V? Does xterm trigger `onData("\x16")`?
// If it triggers `onData("\x16")`, `code === 22`. There is NO HANDLER for `code === 22`.
// If we add `if (data === "\x16")`, we can read from clipboard:
// `navigator.clipboard.readText().then(text => ...)`
// But wait, browsers require user permission or transient activation for `readText`.
// A better way is to attach a custom key handler.
