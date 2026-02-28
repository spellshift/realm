// Okay, so how about we optimize the paste?
// We can check if `data.length > 1 && !data.startsWith("\x1b")` and `data` doesn't contain `\r` or `\n`.
// If it has NO newlines, we can just let it fall through as ONE block!
// The current code handles `data.length > 1` beautifully for normal text (it appends it in one go to `inputBuffer`).
// So we ONLY need to split if `data` CONTAINS `\r` or `\n`!
// If `data` contains `\r` or `\n`, we can split it into chunks:
// `const parts = data.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");`
// Then for each part:
// 1. Append `part` to `inputBuffer`
// 2. If it's not the last part, simulate Enter press!
// Let's implement this!

// ```typescript
// if (data.includes("\r") || data.includes("\n")) {
//    const parts = data.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
//    for (let i = 0; i < parts.length; i++) {
//        const part = parts[i];
//        if (part) {
//            // Append part
//            state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + part + state.inputBuffer.slice(state.cursorPos);
//            state.cursorPos += part.length;
//        }
//        if (i < parts.length - 1) {
//            // Simulate Enter
//            term.write("\r\n");
//            const res = adapter.current?.input(state.inputBuffer);
//            state.currentBlock += state.inputBuffer + "\n";
//            if (res?.status === "complete") {
//                if (state.currentBlock.trim()) {
//                    state.history.push(state.currentBlock.trimEnd());
//                    saveHistory(state.history);
//                }
//                state.currentBlock = "";
//                state.historyIndex = -1;
//                state.inputBuffer = "";
//                state.cursorPos = 0;
//                state.prompt = ">>> ";
//            } else if (res?.status === "incomplete") {
//                state.prompt = res.prompt || ".. ";
//                term.write(state.prompt);
//                state.inputBuffer = "";
//                state.cursorPos = 0;
//            } else {
//                term.write(`Error: ${res?.message}\r\n>>> `);
//                state.currentBlock = "";
//                state.inputBuffer = "";
//                state.cursorPos = 0;
//                state.prompt = ">>> ";
//            }
//            lastBufferHeight.current = 0;
//        }
//    }
//    redrawLine();
//    return;
// }
// ```
// Wait, if I do this, it processes lines instantly and renders once at the end.
// And it gracefully handles multi-line pastes, including `adapter.current?.input` logic!
// Let's refine this plan.
