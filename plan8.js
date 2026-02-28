// Wait! Let's examine if `termInstance.current.attachCustomKeyEventHandler` exists in the codebase.
// Nope.
// To fix this we can add `attachCustomKeyEventHandler` when initializing the terminal.

// ```typescript
// termInstance.current.attachCustomKeyEventHandler(e => {
//    if (e.ctrlKey && e.code === 'KeyC' && e.type === 'keydown') {
//         const selection = termInstance.current.getSelection();
//         if (selection) {
//             navigator.clipboard.writeText(selection);
//             return false;
//         }
//    }
//    if (e.ctrlKey && e.code === 'KeyV' && e.type === 'keydown') {
//         navigator.clipboard.readText().then(text => {
//             // how to paste it into the terminal?
//             // just pass it to the same logic as onData!
//             // Or wait, doesn't xterm have an onPaste event?
//         });
//         return false; // prevent default \x16
//    }
//    return true;
// });
// ```
