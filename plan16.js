// So:
// If `data` is a pasted string like `"Hello\nWorld"`, `code` is 72 ('H').
// It falls into `code >= 32` because it doesn't match any `data === "\x1b..."`.
// If it falls into `code >= 32`, we can split the string by `\r` and `\n` or just loop through characters and process them as if they were individually typed, or handle it as a single block!
// Wait! If the user pastes a large script, splitting by `\r` or `\n` and running Enter logic on each line is the correct terminal behavior!
// Let's replace the `code >= 32` block with:
// ```typescript
// if (code >= 32 && code !== 127) {
//     // If it's a multi-character string that isn't a known escape sequence
//     // Actually, it might contain newlines if it's a paste.
//     if (data.includes('\r') || data.includes('\n')) {
//         // It's a multi-line paste. We should process each line.
//         // However, `data` might also have other control chars?
//         // Best is to process it character by character, BUT recursively calling the handler might be messy.
//         // Better yet:
//         const chars = Array.from(data);
//         // Wait, we can't easily recurse because we modify state inline.
//     }
// }
// ```
