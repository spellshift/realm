// WAIT. If I paste `echo "line1"\r\necho "line2"`, the first part is `echo "line1"`.
// `inputBuffer` gets appended, then `Enter` is simulated.
// Then the second part `echo "line2"` is appended.
// But we only run `redrawLine()` AT THE END.
// Wait, `term.write("\r\n")` moves cursor to next line.
// But we also write `term.write(state.prompt)` or `term.write("Error: ...\r\n>>> ")`.
// Is `redrawLine()` okay?
// Yes, `redrawLine` clears the current input line and rewrites it. Since we only call it at the END, it will correctly redraw the prompt and `inputBuffer` for the LAST line!
// Wait, for the intermediate lines, xterm won't display the typed text unless we manually `term.write(part)` before simulating Enter!
// Yes! If we don't display it, the terminal will just show the prompt and instantly print the output. The typed text won't be visible!
// We should do:
// `term.write(highlightPythonSyntax(part) + "\r\n");`
// Or just reuse the exact `redrawLine` logic for each line? No, `redrawLine` moves cursor UP!
