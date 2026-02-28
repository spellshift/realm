// If we paste multiple characters, what are the codes?
// Actually if we paste "abc", data is "abc".
// "abc".charCodeAt(0) gives the first char code.
// For pasting, if the first character code >= 32, it will hit `code >= 32 && code !== 127`
// However, if we paste multi-line text "a\nb", it will also hit `code >= 32 && code !== 127` because 'a'.charCodeAt(0) == 97.
// But it contains '\n' (or '\r').
// The current logic:
// state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
// state.cursorPos += data.length;
// redrawLine();
