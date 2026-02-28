// Let's create an example of what to add to `termInstance.current`:
// ```typescript
// termInstance.current.attachCustomKeyEventHandler((e) => {
//    if (e.type === "keydown") {
//        if ((e.ctrlKey || e.metaKey) && e.code === "KeyC") {
//            if (termInstance.current?.hasSelection()) {
//                navigator.clipboard.writeText(termInstance.current.getSelection());
//                termInstance.current.clearSelection();
//                return false;
//            }
//        }
//        if ((e.ctrlKey || e.metaKey) && e.code === "KeyV") {
//            navigator.clipboard.readText().then((text) => {
//                 // Wait, xterm already has an internal paste handler that listens to the native `paste` event.
//                 // If the textarea is focused, pressing Ctrl+V triggers the native `paste` event.
//                 // The native `paste` event triggers `onData(text)`.
//                 // BUT if we override `Ctrl+V`, we might double paste or bypass xterm's native paste.
//                 // Let's rely on xterm's native paste event.
//            }).catch(() => {});
//            // return false; // if we do this, it prevents default native paste!
//            // SO WE SHOULD NOT DO THIS FOR KEYV.
//            // The native Ctrl+V works, BUT our `onData` processes it wrong!
//        }
//    }
//    return true;
// });
// ```
