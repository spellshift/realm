import { Terminal } from "@xterm/xterm";
import { MutableRefObject } from "react";
import { ShellState } from "../types";
import { HeadlessWasmAdapter } from "../../../../lib/headless-adapter";
import { moveWordLeft, moveWordRight, saveHistory } from "../shellUtils";
import { CompletionsState } from "../useTerminalCompletions";

interface HandleInputParams {
    data: string;
    term: Terminal | null;
    state: ShellState;
    adapter: HeadlessWasmAdapter | null;
    completionsRef: MutableRefObject<CompletionsState>;
    updateCompletionsUI: (list: string[], start: number, show: boolean, index: number) => void;
    applyCompletion: (completion: string) => void;
    redrawLine: () => void;
    lastBufferHeight: MutableRefObject<number>;
    scheduleRedraw: () => void;
}

export const handleTerminalInput = ({
    data,
    term,
    state,
    adapter,
    completionsRef,
    updateCompletionsUI,
    applyCompletion,
    redrawLine,
    lastBufferHeight,
    scheduleRedraw
}: HandleInputParams) => {
    if (!term) return;

    const code = data.charCodeAt(0);

    // If completions are showing, handle navigation
    if (completionsRef.current.show) {
        if (code === 9) { // Tab: cycle
            const list = completionsRef.current.list;
            if (list.length > 0) {
                const next = (completionsRef.current.index + 1) % list.length;
                updateCompletionsUI(list, completionsRef.current.start, true, next);
            }
            return;
        }
        if (code === 13) { // Enter: select
            applyCompletion(completionsRef.current.list[completionsRef.current.index]);
            return;
        }
        if (data === "\x1b[B") { // Down
            const list = completionsRef.current.list;
            if (list.length > 0) {
                const next = (completionsRef.current.index + 1) % list.length;
                updateCompletionsUI(list, completionsRef.current.start, true, next);
            }
            return;
        }
        if (data === "\x1b[A") { // Up
            const list = completionsRef.current.list;
            if (list.length > 0) {
                const next = (completionsRef.current.index - 1 + list.length) % list.length;
                updateCompletionsUI(list, completionsRef.current.start, true, next);
            }
            return;
        }
        if (code === 27) { // Esc: cancel
            updateCompletionsUI([], 0, false, 0);
            return;
        }
        // Allow other keys to fall through to input handler
    }

    if (state.isSearching) {
        if (data === "\x12") { // Ctrl+R (Next match)
            // Logic to find next match (skipping current index?)
            return;
        }
        if (data === "\x03" || data === "\x07") { // Ctrl+C / Ctrl+G
            state.isSearching = false;
            state.searchQuery = "";
            redrawLine();
            return;
        }
        if (code === 13) { // Enter
            // Use the match
            state.isSearching = false;
            let match = "";
            if (state.searchQuery) {
                for (let i = state.history.length - 1; i >= 0; i--) {
                    if (state.history[i].includes(state.searchQuery)) {
                        match = state.history[i];
                        break;
                    }
                }
            }
            state.inputBuffer = match;
            state.cursorPos = match.length;
            state.searchQuery = "";
            redrawLine();
            return;
        }
        if (code === 127) { // Backspace
            state.searchQuery = state.searchQuery.slice(0, -1);
            redrawLine();
            return;
        }
        if (code >= 32) {
            state.searchQuery += data;
            redrawLine();
            return;
        }
        // Ignore other keys in search mode for now
        return;
    }

    // Normal mode
    if (data === "\x03") { // Ctrl+C
        adapter?.reset();
        term.write("^C\r\n");
        state.inputBuffer = "";
        state.currentBlock = "";
        state.cursorPos = 0;
        state.historyIndex = -1;
        state.prompt = ">>> ";
        term.write(state.prompt);
        lastBufferHeight.current = 0;
        return;
    }

    if (data === "\x12") { // Ctrl+R
        state.isSearching = true;
        state.searchQuery = "";
        redrawLine();
        return;
    }

    if (data === "\x0c") { // Ctrl+L
        term.write('\x1b[2J\x1b[H');
        lastBufferHeight.current = 0;
        redrawLine();
        return;
    }

    if (data === "\x01" || data === "\x1b[H" || data === "\x1bOH") { // Ctrl+A / Home
        state.cursorPos = 0;
        redrawLine();
        return;
    }

    if (data === "\x05" || data === "\x1b[F" || data === "\x1bOF") { // Ctrl+E / End
        state.cursorPos = state.inputBuffer.length;
        redrawLine();
        return;
    }

    if (data === "\x15") { // Ctrl+U
        state.inputBuffer = "";
        state.cursorPos = 0;
        redrawLine();
        return;
    }

    if (data === "\x0b") { // Ctrl+K (Kill to end)
        state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos);
        redrawLine();
        return;
    }

    // Word Navigation: Left (Alt+Left, Alt+b)
    if (data === "\x1b[1;3D" || data === "\x1b[1;5D" || data === "\x1b\x1b[D" || data === "\x1bb") {
        state.cursorPos = moveWordLeft(state.inputBuffer, state.cursorPos);
        redrawLine();
        return;
    }

    // Word Navigation: Right (Alt+Right, Alt+f)
    if (data === "\x1b[1;3C" || data === "\x1b[1;5C" || data === "\x1b\x1b[C" || data === "\x1bf") {
        state.cursorPos = moveWordRight(state.inputBuffer, state.cursorPos);
        redrawLine();
        return;
    }

    // Word Deletion: Backward (Alt+Backspace / Ctrl+W)
    if (data === "\x17" || data === "\x1b\x7f" || data === "\x1b\x08") {
        const newPos = moveWordLeft(state.inputBuffer, state.cursorPos);
        const afterCursor = state.inputBuffer.slice(state.cursorPos);
        state.inputBuffer = state.inputBuffer.slice(0, newPos) + afterCursor;
        state.cursorPos = newPos;
        redrawLine();
        return;
    }

    // Word Deletion: Forward (Alt+Delete / Alt+d)
    if (data === "\x1b[3;3~" || data === "\x1bd") {
        const endPos = moveWordRight(state.inputBuffer, state.cursorPos);
        const beforeCursor = state.inputBuffer.slice(0, state.cursorPos);
        const afterWord = state.inputBuffer.slice(endPos);
        state.inputBuffer = beforeCursor + afterWord;
        // Cursor stays at current pos
        redrawLine();
        return;
    }

    if (data === "\x1b[A") { // Up
        if (state.history.length > 0) {
            if (state.historyIndex === -1) state.historyIndex = state.history.length - 1;
            else if (state.historyIndex > 0) state.historyIndex--;
            state.inputBuffer = state.history[state.historyIndex];
            state.cursorPos = state.inputBuffer.length;
            redrawLine();
        }
        return;
    }

    if (data === "\x1b[B") { // Down
        if (state.historyIndex !== -1) {
            if (state.historyIndex < state.history.length - 1) {
                state.historyIndex++;
                state.inputBuffer = state.history[state.historyIndex];
            } else {
                state.historyIndex = -1;
                state.inputBuffer = "";
            }
            state.cursorPos = state.inputBuffer.length;
            redrawLine();
        }
        return;
    }

    if (data === "\x1b[D") { // Left
        if (state.cursorPos > 0) {
            state.cursorPos--;
            term.write("\x1b[D");
        }
        return;
    }

    if (data === "\x1b[C") { // Right
        if (state.cursorPos < state.inputBuffer.length) {
            state.cursorPos++;
            term.write("\x1b[C");
        }
        return;
    }

    if (code === 9) { // Tab
        // Indent if line is empty or whitespace
        if (!state.inputBuffer.trim()) {
            const indent = "    ";
            state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + indent + state.inputBuffer.slice(state.cursorPos);
            state.cursorPos += 4;
            term.write(indent);
            return;
        }

        // Trigger completion
        const res = adapter?.complete(state.inputBuffer, state.cursorPos);
        if (res && res.suggestions.length > 0) {
            if (res.suggestions.length === 1) {
                // Auto complete
                const completion = res.suggestions[0];
                const start = res.start;
                if (start >= 0 && start <= state.cursorPos) {
                    const prefix = state.inputBuffer.slice(0, start);
                    const suffix = state.inputBuffer.slice(state.cursorPos);
                    state.inputBuffer = prefix + completion + suffix;
                    state.cursorPos = start + completion.length;
                    redrawLine();
                }
            } else {
                // Show dropdown
                updateCompletionsUI(res.suggestions, res.start, true, 0);
            }
        }
        return;
    }

    if (code >= 32 && code !== 127) {
        // Fast path for simple appending at the end of the line
        if (data.length === 1 && state.cursorPos === state.inputBuffer.length) {
            state.inputBuffer += data;
            state.cursorPos++;
            term.write(data);
            scheduleRedraw();
        } else {
            state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
            state.cursorPos += data.length;
            redrawLine();
        }
    } else if (code === 13) { // Enter
        term.write("\r\n");
        const res = adapter?.input(state.inputBuffer);

        state.currentBlock += state.inputBuffer + "\n";

        if (res?.status === "complete") {
            if (state.currentBlock.trim()) {
                state.history.push(state.currentBlock.trimEnd());
                saveHistory(state.history);
            }
            state.currentBlock = "";
            state.historyIndex = -1;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
        } else if (res?.status === "incomplete") {
            state.prompt = res.prompt || ".. ";
            term.write(state.prompt);
            state.inputBuffer = "";
            state.cursorPos = 0;
        } else {
            term.write(`Error: ${res?.message}\r\n>>> `);
            state.currentBlock = "";
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
        }
        lastBufferHeight.current = 0;
    } else if (code === 127) { // Backspace
        if (state.cursorPos > 0) {
            // Fast path for backspace at the end of the line
            if (state.cursorPos === state.inputBuffer.length) {
                state.inputBuffer = state.inputBuffer.slice(0, -1);
                state.cursorPos--;
                term.write('\b \b');
                scheduleRedraw();
            } else {
                state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos - 1) + state.inputBuffer.slice(state.cursorPos);
                state.cursorPos--;
                redrawLine();
            }
        }
    }

    // Trigger completion updates if needed
    if (completionsRef.current.show || code === 46 /* . */) {
        const res = adapter?.complete(state.inputBuffer, state.cursorPos);
        if (res && res.suggestions.length > 0) {
            updateCompletionsUI(res.suggestions, res.start, true, 0);
        } else {
            if (completionsRef.current.show) {
                updateCompletionsUI([], 0, false, 0);
            }
        }
    }
};
