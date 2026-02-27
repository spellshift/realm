import { Terminal } from "@xterm/xterm";
import { MutableRefObject } from "react";
import { ShellState } from "../types";
import { highlightPythonSyntax } from "../shellUtils";

export const renderLine = (
    term: Terminal | null,
    state: ShellState,
    lastBufferHeight: MutableRefObject<number>
) => {
    if (!term) return;

    let contentToWrite = "";
    let contentToDisplay = "";
    let cursorIndex = 0;

    if (state.isSearching) {
        const prompt = `(reverse-i-search)'${state.searchQuery}': `;
        let match = "";
        if (state.searchQuery) {
            for (let i = state.history.length - 1; i >= 0; i--) {
                if (state.history[i].includes(state.searchQuery)) {
                    match = state.history[i];
                    break;
                }
            }
        }
        contentToWrite = prompt + match;
        contentToDisplay = contentToWrite;
        cursorIndex = contentToWrite.length;
    } else {
        contentToWrite = state.prompt + state.inputBuffer;
        contentToDisplay = state.prompt + highlightPythonSyntax(state.inputBuffer);
        cursorIndex = state.prompt.length + state.cursorPos;
    }

    const termCols = term.cols;
    const getVisualLineCount = (text: string, cols: number) => {
        const lines = text.split('\n');
        let count = 0;
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            if (i > 0) count++;
            if (line.length > 0) {
                count += Math.floor((line.length - 1) / cols);
            }
        }
        return count;
    };

    const rows = getVisualLineCount(contentToWrite, termCols);

    const prevRows = lastBufferHeight.current;
    if (prevRows > 0) {
        term.write(`\x1b[${prevRows}A`);
    }

    term.write("\r\x1b[J");

    term.write(contentToDisplay.replace(/\n/g, "\r\n"));

    lastBufferHeight.current = rows;

    if (!state.isSearching) {
        const prefix = contentToWrite.slice(0, cursorIndex);
        const cursorRow = getVisualLineCount(prefix, termCols);
        const lastLine = prefix.split('\n').pop() || "";
        let cursorCol = lastLine.length % termCols;

        const moveUp = rows - cursorRow;
        if (moveUp > 0) {
            term.write(`\x1b[${moveUp}A`);
        }

        term.write("\r");
        if (cursorCol > 0) {
            term.write(`\x1b[${cursorCol}C`);
        }
    }
};
