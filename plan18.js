// So here is my plan:
// 1. Modify `useShellTerminal.ts` inside `termInstance.current.onData((data) => {`
//    A. Fix Ctrl+C (Copy instead of SIGINT if there's text selected)
//       ```typescript
//       if (data === "\x03") { // Ctrl+C
//           if (term.hasSelection()) {
//               navigator.clipboard.writeText(term.getSelection());
//               term.clearSelection();
//               return;
//           }
//           ... // proceed with reset
//       }
//       ```
//    B. Process pasted string gracefully by checking if `data` contains multi-character text and is not a control sequence, or just looping through characters.
//       Wait, what if `data` is `\r` (just Enter)? `charCodeAt(0)` is 13.
//       Actually, the cleanest way to handle arbitrary pasted text is to parse it, BUT xterm usually sends it as one big string if the browser native paste happens.
//       If we just do this at the very beginning of `onData(data)`:
//       ```typescript
//       // If this is a pasted chunk of text (not a single keypress and not an escape sequence)
//       if (data.length > 1 && !data.startsWith("\x1b")) {
//           // Normalize newlines to \r
//           const normalized = data.replace(/\r\n/g, "\r").replace(/\n/g, "\r");
//           for (let i = 0; i < normalized.length; i++) {
//               // We can't just call onData(normalized[i]) because xterm passes it, but we COULD extract the handler into a function!
//           }
//       }
//       ```
//       Yes! We can extract the inner body of `onData` to a function `processData(char)` or we can just iterate over `normalized.split('')` calling the function.
//       Wait, if we extract the inner body into a function:
//       ```typescript
//       const handleData = (chunk: string) => { ... }
//       termInstance.current.onData((data) => {
//           if (data.length > 1 && !data.startsWith("\x1b")) {
//               const normalized = data.replace(/\r\n/g, "\r").replace(/\n/g, "\r");
//               for (const char of normalized) {
//                   handleData(char);
//               }
//           } else {
//               handleData(data);
//           }
//       });
//       ```
//       Wait, does a multi-character paste start with `\x1b`? Only if the user copied an escape sequence! But usually they don't.
//       If they DO, it's fine. What if `data` is `\x1b[A`? It goes to the `else` block (`handleData(data)`), which processes it as Arrow Up! Perfect!
//       What if they paste "a\r\nb\r\n"?
//       It processes 'a', '\r', 'b', '\r'.
//       Wait! `data.length > 1 && !data.startsWith("\x1b")`? What if they paste text that starts with `\x1b`? Let's just say we don't handle copying escape sequences for pasting text.
