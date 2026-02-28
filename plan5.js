// Wait! Let's verify `xterm` configuration.
// By default, `macOptionIsMeta: true` is set.
// To handle Ctrl+C / Cmd+C for copy:
// `term.attachCustomKeyEventHandler` can be used to intercept keys.
// If there's a selection (`term.hasSelection()`), Ctrl+C should copy.
// Wait, doesn't xterm have a plugin or native support for `Ctrl+C` / `Ctrl+V`?
// Actually, `term.attachCustomKeyEventHandler(e => { ... })`
// Let's create a test node script to mock what we'd add to `useShellTerminal.ts`.
