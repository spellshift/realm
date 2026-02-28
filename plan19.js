// WAIT, the `handleData` function processes one character at a time.
// If we process it character by character, then `scheduleRedraw()` gets called 50 times for a 50 char paste!
// Actually, `scheduleRedraw()` uses `setTimeout` to debounce the redraws! So it's very efficient!
// Let's look at `scheduleRedraw`:
// ```typescript
// const scheduleRedraw = () => {
//    if (redrawTimeoutRef.current) {
//        clearTimeout(redrawTimeoutRef.current);
//    }
//    redrawTimeoutRef.current = setTimeout(() => {
//        redrawLine();
//    }, 50);
// };
// ```
// If we loop through 50 characters, `state.inputBuffer` gets appended 50 times synchronously.
// `redrawLine` is only called once after 50ms!
// BUT wait, in the loop:
// ```typescript
// if (code >= 32 && code !== 127) {
//    // Fast path for simple appending at the end of the line
//    if (data.length === 1 && state.cursorPos === state.inputBuffer.length) {
//        state.inputBuffer += data;
//        state.cursorPos++;
//        term.write(data);
//        scheduleRedraw();
//    } else {
//        state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + ...
//        state.cursorPos += data.length;
//        redrawLine(); // OOPS! This calls redrawLine synchronously!
//    }
// }
// ```
// If `data.length === 1` and `cursorPos === inputBuffer.length` (which is true when pasting at the end of the line character by character), it uses the fast path and debounces redraw!
// BUT if we paste in the MIDDLE of a line, it calls `redrawLine()` synchronously for every character!
// This might be slow for a 10,000 character paste!
