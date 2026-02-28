// Actually, if we just extract the processing logic into a function `processInput(chunk)` and call it?
// Or we can just handle the whole paste in one go!
// If `data` contains newlines, we can do:
// ```typescript
// const parts = data.split(/\r\n|\r|\n/);
// for (let i = 0; i < parts.length; i++) {
//     const part = parts[i];
//     if (part.length > 0) {
//         state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + part + state.inputBuffer.slice(state.cursorPos);
//         state.cursorPos += part.length;
//     }
//     if (i < parts.length - 1) {
//         // simulate Enter
//         term.write("\r\n");
//         const res = adapter.current?.input(state.inputBuffer);
//         state.currentBlock += state.inputBuffer + "\n";
//         // ... handle res ...
//     }
// }
// redrawLine();
// ```
// Yes! This perfectly handles pasting multiple lines!
// And if `data` does not contain newlines, `parts.length` is 1, and it just appends the part normally!
// BUT what if `data` is exactly `"\r"` or `"\n"` or `"\r\n"`?
// If `data` is `"\r"`, it falls under `code === 13`.
// If `data === "\n"`, code is 10. `10 >= 32` is false!
// Where is `code === 10` handled? It's not!
// So `\n` is ignored!
