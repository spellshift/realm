// By default, xterm handles copying automatically if you use standard browser copy commands.
// BUT `Ctrl+C` sends \x03 because it's a terminal.
// For pasting, `Ctrl+V` sends \x16.
// Also, `Cmd+C` / `Cmd+V` on Mac works automatically because they don't send terminal control characters.
// The issue "Copy + Paste does not work" likely means on Windows/Linux, Ctrl+C / Ctrl+V don't work.
// To fix this, we can intercept Ctrl+C / Ctrl+V in `attachCustomKeyEventHandler`.
