// Wait! xterm handles pasting natively with `Shift+Insert` or browser Context Menu -> Paste, or Command+V on Mac.
// When pasting via browser's native features, it triggers the `paste` event on the terminal's textarea, and xterm passes the text to `onData`.
// But xterm.js also has `onPaste`!
// Let's check the API:
// termInstance.current.onPaste((data) => { ... });
// But wait, if we use `onPaste`, the pasted text ALSO fires `onData`!
// Wait! If `onData` receives the pasted text, our current logic handles it as ONE large string in `data`.
// If `data` contains newlines (e.g. `echo 1\necho 2\n`), it gets dumped into `inputBuffer`!
// AND if the user manually pastes multiple commands, they aren't executed until they press Enter!
// AND what if they paste text containing control characters?
