// WAIT, the `code >= 32` block handles `data.length > 1` if it has NO newlines:
// ```typescript
// if (data.length === 1 && state.cursorPos === state.inputBuffer.length) {
//    ...
// } else {
//    state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + ...
//    state.cursorPos += data.length;
//    redrawLine();
// }
// ```
// If `data` is a pasted string without newlines, it falls here, and works perfectly!
// So we JUST need to handle `data` with newlines!
// Where should we put this logic?
// Right at the start of `onData`, after checking connection status!

// ```typescript
// termInstance.current.onData((data) => {
//    if (isLateCheckinRef.current) return;
//    if (connectionStatusRef.current !== "connected") return;

//    // Handle Ctrl+C copy
//    if (data === "\x03" && termInstance.current?.hasSelection()) {
//        navigator.clipboard.writeText(termInstance.current.getSelection());
//        termInstance.current.clearSelection();
//        return;
//    }

//    // Handle multi-line paste
//    if (data.length > 1 && !data.includes("\x1b") && (data.includes("\r") || data.includes("\n"))) {
//        const parts = data.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
//        const term = termInstance.current;
//        const state = shellState.current;
//        if (!term) return;

//        for (let i = 0; i < parts.length; i++) {
//            const part = parts[i];
//            if (part) {
//                state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + part + state.inputBuffer.slice(state.cursorPos);
//                state.cursorPos += part.length;
//            }
//
//            if (i < parts.length - 1) {
//                // Redraw to show what was pasted on this line
//                redrawLine();
//                // Simulate Enter
//                term.write("\r\n");
//                const res = adapter.current?.input(state.inputBuffer);
//                state.currentBlock += state.inputBuffer + "\n";
//                if (res?.status === "complete") {
//                    if (state.currentBlock.trim()) {
//                        state.history.push(state.currentBlock.trimEnd());
//                        saveHistory(state.history);
//                    }
//                    state.currentBlock = "";
//                    state.historyIndex = -1;
//                    state.inputBuffer = "";
//                    state.cursorPos = 0;
//                    state.prompt = ">>> ";
//                } else if (res?.status === "incomplete") {
//                    state.prompt = res.prompt || ".. ";
//                    term.write(state.prompt);
//                    state.inputBuffer = "";
//                    state.cursorPos = 0;
//                } else {
//                    term.write(`Error: ${res?.message}\r\n>>> `);
//                    state.currentBlock = "";
//                    state.inputBuffer = "";
//                    state.cursorPos = 0;
//                    state.prompt = ">>> ";
//                }
//                lastBufferHeight.current = 0;
//            }
//        }
//        redrawLine();
//        return;
//    }
// ```
// This plan fixes BOTH Copy (Ctrl+C on Windows/Linux) and Paste (multi-line commands).
