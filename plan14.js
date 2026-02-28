// Let's look at `data` again.
// If the user pastes `"a\r\nb"`, `data` might be exactly that.
// If we process it character by character in a loop, it would trigger `data === "\r"` which is `code === 13`.
// Then we run the `adapter.current?.input` logic!
// BUT wait, `adapter.current?.input` might be asynchronous? No, it returns `{status: "complete", ...}` synchronously.
// If we process character by character:
// ```typescript
// const chars = [...data];
// for (const char of chars) {
//     const code = char.charCodeAt(0);
//     // ... process char
// }
// ```
// This would run the Enter logic multiple times.
// But wait, what if `data` is a huge string? Processing character by character might be slow.
// Actually, it's better to process pasted data correctly!
