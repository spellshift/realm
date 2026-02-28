// If we process lines:
// ```typescript
// if (data.length > 1 && !data.includes("\x1b") && (data.includes("\r") || data.includes("\n"))) {
//     const parts = data.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
//     // ... process parts ...
//     return;
// }
// ```
// If it has NO newlines, we can just let it fall through `code >= 32` where `state.inputBuffer = ... + data`.
// Then we just `redrawLine()` ONCE. This is very efficient for single line pastes!
// And for multi-line pastes, we split by `\n`, process each part:
// 1. `state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + part + ...`
// 2. `state.cursorPos += part.length;`
// 3. For all parts except the last, simulate `Enter`:
//    `redrawLine()`
//    `term.write("\r\n")`
//    `adapter.current?.input(...)`
// 4. For the last part, just `redrawLine()`.
// This avoids character-by-character processing, preserves escape sequences, AND renders perfectly!
