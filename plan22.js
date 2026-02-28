// Okay, so if I simulate Enter:
// `term.write("\r\n")` is executed in `else if (code === 13)`.
// But BEFORE that, the user typed `echo 1`. `redrawLine()` was called, so `>>> echo 1` is displayed.
// When Enter is pressed, `term.write("\r\n")` moves to next line.
// Then `adapter.current?.input` happens, and output is printed by the websocket handler asynchronously!
// BUT if it's an incomplete block, it prints `.. ` synchronously!
// Wait! If `adapter.current?.input` is called for `echo 1`, the backend responds via websocket, and `adapter.current` prints the output using `msg.output` -> `term.write()`.
// This is fully asynchronous!
// So if I process a block of 3 lines:
// Line 1 -> `adapter.current?.input(line1)`
// Line 2 -> `adapter.current?.input(line2)`
// Line 3 -> `adapter.current?.input(line3)`
// The output from Line 1 will arrive LATER!
// This means the terminal might look like:
// >>> echo 1
// >>> echo 2
// >>> echo 3
// (Wait, I didn't print `>>> echo 1`, I only appended `inputBuffer` internally and ran `adapter.current?.input`!)
// If I don't call `redrawLine()` for each line, the terminal will literally NOT SHOW `>>> echo 1`.
// It will just be invisible!
// And then the output for `echo 1` will appear!
