// So the best way is to extract `termInstance.current.onData` logic to a function `handleChar(data: string)` and call it for EACH character.
// Wait, if I call it for each character, `redrawLine` is debounced!
// So it will only draw once at the end? NO, `redrawLine` is synchronous in `else { redrawLine() }`!
// Wait! `scheduleRedraw()` is ONLY used for fast path `data.length === 1 && state.cursorPos === state.inputBuffer.length`.
// If I iterate over each character:
// ```typescript
// for (let i = 0; i < data.length; i++) {
//    handleChar(data[i]);
// }
// ```
// For EACH character:
// 1. `code >= 32` matches. `data.length === 1` is true! `cursorPos === inputBuffer.length` is true!
// 2. `inputBuffer += char`.
// 3. `term.write(char)`!
// 4. `scheduleRedraw()`!
// This is perfect! `term.write(char)` instantly prints the character without moving cursor up or clearing the line!
// Then if the next character is `\n` or `\r` (`13`), `handleChar("\r")` is called.
// It executes the `code === 13` block!
// 1. `term.write("\r\n")` (moves cursor to next line)
// 2. `adapter.current?.input()`
// 3. Resets `inputBuffer`. Prints `>>> `!
// This is exactly how typing behaves!
// AND it handles everything natively, exactly as if the user typed it really fast!
