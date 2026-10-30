import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
    moveWordLeft,
    moveWordRight,
    getCursorLineInfo,
    moveCursorUpInBuffer,
    moveCursorDownInBuffer,
    moveCursorToLineStart,
    moveCursorToLineEnd,
    dedentCurrentLine,
    hasUnclosedBracketsOrStrings,
    shouldContinueMultilineOnEnter,
    insertNewlineWithAutoIndent,
    formatMultilineTerminalBuffer,
    getTerminalCursorLayout,
    buildTerminalRedrawSequence,
    highlightPythonSyntax,
    loadHistory,
    saveHistory,
    buildScrollbackPreservingClearSequence,
    transformClearAnsiForScrollback,
    getViewportNonEmptyLineCount,
    HISTORY_KEY,
    MAX_HISTORY,
} from './shellUtils';

describe('History Persistence', () => {
    beforeEach(() => {
        if (typeof globalThis.localStorage === 'undefined' || typeof globalThis.localStorage?.clear !== 'function') {
            const store = new Map<string, string>();
            Object.defineProperty(globalThis, 'localStorage', {
                configurable: true,
                writable: true,
                value: {
                    getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
                    setItem: (k: string, v: string) => store.set(k, String(v)),
                    removeItem: (k: string) => store.delete(k),
                    clear: () => store.clear(),
                },
            });
        }
        // Clear localStorage before each test
        localStorage.clear();
        vi.restoreAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it('loadHistory returns empty array when storage is empty', () => {
        expect(loadHistory()).toEqual([]);
    });

    it('loadHistory returns parsed array from storage', () => {
        const history = ["cmd1", "cmd2"];
        localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
        expect(loadHistory()).toEqual(history);
    });

    it('loadHistory returns empty array on parse error', () => {
        localStorage.setItem(HISTORY_KEY, "invalid json");
        // Mock console.error to suppress expected error log
        const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
        expect(loadHistory()).toEqual([]);
        expect(consoleSpy).toHaveBeenCalled();
    });

    it('saveHistory saves data to localStorage', () => {
        const history = ["cmd1", "cmd2"];
        saveHistory(history);
        const stored = localStorage.getItem(HISTORY_KEY);
        expect(JSON.parse(stored!)).toEqual(history);
    });

    it('saveHistory truncates history exceeding MAX_HISTORY', () => {
        const history = Array.from({ length: MAX_HISTORY + 10 }, (_, i) => `cmd${i}`);
        saveHistory(history);
        const stored = JSON.parse(localStorage.getItem(HISTORY_KEY)!);
        expect(stored.length).toBe(MAX_HISTORY);
        // Should keep the last MAX_HISTORY items
        expect(stored[0]).toBe('cmd10');
        expect(stored[MAX_HISTORY - 1]).toBe(`cmd${MAX_HISTORY + 9}`);
    });
});

describe('moveWordLeft', () => {
    it('moves to start of current word', () => {
        const buffer = "hello world";
        // "hello world|"
        expect(moveWordLeft(buffer, 11)).toBe(6); // "hello |world"
    });

    it('moves to start of previous word', () => {
        const buffer = "hello world";
        // "hello |world" (6)
        expect(moveWordLeft(buffer, 6)).toBe(0); // "|hello world"
    });

    it('handles multiple spaces', () => {
        const buffer = "a   b";
        // "a   b|" (5)
        expect(moveWordLeft(buffer, 5)).toBe(4); // "a   |b"
        // "a   |b" (4)
        expect(moveWordLeft(buffer, 4)).toBe(0); // "|a   b"
    });

    it('handles empty/single word', () => {
        const buffer = "hello";
        expect(moveWordLeft(buffer, 5)).toBe(0);
        expect(moveWordLeft(buffer, 0)).toBe(0);
    });

    it('handles cursor in middle of word', () => {
        const buffer = "hello";
        expect(moveWordLeft(buffer, 3)).toBe(0);
    });
});

describe('moveWordRight', () => {
    it('moves to end of current word', () => {
        const buffer = "hello world";
        // "|hello world" (0)
        expect(moveWordRight(buffer, 0)).toBe(5); // "hello| world"
    });

    it('moves to end of next word', () => {
        const buffer = "hello world";
        // "hello| world" (5)
        expect(moveWordRight(buffer, 5)).toBe(11); // "hello world|"
    });

    it('handles multiple spaces', () => {
        const buffer = "a   b";
        // "|a   b" (0)
        expect(moveWordRight(buffer, 0)).toBe(1); // "a|   b"
        // "a|   b" (1)
        expect(moveWordRight(buffer, 1)).toBe(5); // "a   b|"
    });

    it('handles cursor in middle of word', () => {
        const buffer = "hello";
        expect(moveWordRight(buffer, 2)).toBe(5);
    });

    it('handles end of buffer', () => {
        const buffer = "hello";
        expect(moveWordRight(buffer, 5)).toBe(5);
    });
});

describe('Multi-line terminal buffer editing', () => {
    it('navigates up and down between lines in a multi-line buffer to fix typos', () => {
        const line0 = 'def my_func():';
        const line1 = '   print("oops a typo)';
        const line2 = 'print("now i\'m here, but I need to go up to fix my input")';
        const buffer = `${line0}\n${line1}\n${line2}`;

        // Start at end of line 2
        const endCursor = buffer.length;
        const info2 = getCursorLineInfo(buffer, endCursor);
        expect(info2.lineIndex).toBe(2);
        expect(info2.lines.length).toBe(3);

        // Move Up to line 1 (clamped to end of line 1)
        const upToLine1 = moveCursorUpInBuffer(buffer, endCursor);
        expect(upToLine1).not.toBeNull();
        const info1 = getCursorLineInfo(buffer, upToLine1!);
        expect(info1.lineIndex).toBe(1);
        expect(info1.col).toBe(line1.length);

        // Move Up to line 0 (clamped to end of line 0)
        const upToLine0 = moveCursorUpInBuffer(buffer, upToLine1!);
        expect(upToLine0).not.toBeNull();
        const info0 = getCursorLineInfo(buffer, upToLine0!);
        expect(info0.lineIndex).toBe(0);
        expect(info0.col).toBe(line0.length);

        // At top line (line 0), moveCursorUpInBuffer returns null so command history can take over
        expect(moveCursorUpInBuffer(buffer, upToLine0!)).toBeNull();

        // Move Down back to line 1 and line 2
        const downToLine1 = moveCursorDownInBuffer(buffer, upToLine0!);
        expect(downToLine1).not.toBeNull();
        expect(getCursorLineInfo(buffer, downToLine1!).lineIndex).toBe(1);

        const downToLine2 = moveCursorDownInBuffer(buffer, downToLine1!);
        expect(downToLine2).not.toBeNull();
        expect(getCursorLineInfo(buffer, downToLine2!).lineIndex).toBe(2);

        // At bottom line (line 2), moveCursorDownInBuffer returns null
        expect(moveCursorDownInBuffer(buffer, downToLine2!)).toBeNull();
    });

    it('moves cursor to start and end of current line only in a multi-line buffer', () => {
        const buffer = 'def foo():\n    x = 1\n    return x';
        // Cursor in middle of line 1 ("    x| = 1")
        const cursor = 'def foo():\n    x'.length;
        expect(moveCursorToLineStart(buffer, cursor)).toBe('def foo():\n'.length);
        expect(moveCursorToLineEnd(buffer, cursor)).toBe('def foo():\n    x = 1'.length);
    });

    it('continues multi-line editing on Enter when block is open or cursor is on an earlier line, and submits on trailing blank line', () => {
        // Colon at end of single line -> continue
        expect(shouldContinueMultilineOnEnter('def my_func():', 14)).toBe(true);

        // Inside multi-line block with non-empty last line -> continue
        const block = 'def my_func():\n    print("hi")';
        expect(shouldContinueMultilineOnEnter(block, block.length)).toBe(true);

        // Editing an earlier line (line 0) when last line is blank -> still continue (don't submit mid-edit)
        const blockWithBlankEnd = 'def my_func():\n    print("hi")\n';
        expect(shouldContinueMultilineOnEnter(blockWithBlankEnd, 5)).toBe(true);

        // Pressing Enter at the trailing blank line -> submit (returns false)
        expect(shouldContinueMultilineOnEnter(blockWithBlankEnd, blockWithBlankEnd.length)).toBe(false);
    });

    it('auto-indents newlines after colons and preserves indentation', () => {
        const res1 = insertNewlineWithAutoIndent('def my_func():', 14);
        expect(res1.buffer).toBe('def my_func():\n    ');
        expect(res1.cursorPos).toBe(res1.buffer.length);

        const res2 = insertNewlineWithAutoIndent(res1.buffer + 'x = 1', res1.buffer.length + 5);
        expect(res2.buffer).toBe('def my_func():\n    x = 1\n    ');
    });

    it('preserves preferredCol across shorter or empty intermediate lines on Up/Down navigation', () => {
        const buffer = 'def my_function(alpha, beta):\n    \n    return alpha + beta';
        const endCursor = buffer.length;
        const startCol = getCursorLineInfo(buffer, endCursor).col; // 23

        const up1 = moveCursorUpInBuffer(buffer, endCursor, startCol)!;
        expect(getCursorLineInfo(buffer, up1)).toMatchObject({ lineIndex: 1, col: 4 });

        const up2 = moveCursorUpInBuffer(buffer, up1, startCol)!;
        expect(getCursorLineInfo(buffer, up2)).toMatchObject({ lineIndex: 0, col: startCol });

        const down1 = moveCursorDownInBuffer(buffer, up2, startCol)!;
        expect(getCursorLineInfo(buffer, down1)).toMatchObject({ lineIndex: 1, col: 4 });

        const down2 = moveCursorDownInBuffer(buffer, down1, startCol)!;
        expect(getCursorLineInfo(buffer, down2)).toMatchObject({ lineIndex: 2, col: startCol });
    });

    it('dedents current line by up to 4 spaces on Shift+Tab and respects newlines in word movement', () => {
        const buffer = 'def foo():\n    return 1';
        const cursor = buffer.length;
        const dedented = dedentCurrentLine(buffer, cursor);
        expect(dedented.buffer).toBe('def foo():\nreturn 1');
        expect(dedented.cursorPos).toBe('def foo():\nreturn 1'.length);

        // Word left at start of unindented second line should stop at start of word or line, not eat across newline
        const afterReturn = 'def foo():\nreturn'.length;
        expect(moveWordLeft('def foo():\nreturn', afterReturn)).toBe('def foo():\n'.length);
        expect(moveWordRight('def foo():\nreturn', 4)).toBe('def foo():'.length);
    });

    it('tracks cursorRow accurately across multi-line Up/Down navigation and edits without overwriting history lines', () => {
        // Minimal VT100/xterm screen simulator that processes \r, \n, \x1b[nA, \x1b[nC, and \x1b[J
        const createTerminalSimulator = (cols: number) => {
            const lines: string[] = [''];
            let row = 0;
            let col = 0;

            const stripAnsiColors = (s: string) =>
                s.replace(/\x1b\[[0-9;]*m/g, '');

            const write = (rawData: string) => {
                const data = stripAnsiColors(rawData);
                let i = 0;
                while (i < data.length) {
                    if (data[i] === '\x1b' && data[i + 1] === '[') {
                        let j = i + 2;
                        while (j < data.length && /[0-9;]/.test(data[j])) {
                            j++;
                        }
                        const paramStr = data.slice(i + 2, j);
                        const cmd = data[j];
                        const count = paramStr ? parseInt(paramStr, 10) : 1;
                        if (cmd === 'A') {
                            row = Math.max(0, row - count);
                        } else if (cmd === 'B') {
                            row += count;
                            while (lines.length <= row) lines.push('');
                        } else if (cmd === 'C') {
                            col = Math.min(cols - 1, col + count);
                        } else if (cmd === 'D') {
                            col = Math.max(0, col - count);
                        } else if (cmd === 'J') {
                            lines[row] = (lines[row] ?? '').slice(0, col);
                            lines.splice(row + 1);
                        }
                        i = j + 1;
                        continue;
                    }
                    const ch = data[i];
                    if (ch === '\r') {
                        col = 0;
                    } else if (ch === '\n') {
                        row++;
                        while (lines.length <= row) lines.push('');
                    } else {
                        if (col >= cols) {
                            row++;
                            col = 0;
                            while (lines.length <= row) lines.push('');
                        }
                        const cur = lines[row] ?? '';
                        const padded = cur.length < col ? cur.padEnd(col, ' ') : cur;
                        lines[row] = padded.slice(0, col) + ch + padded.slice(col + 1);
                        col++;
                    }
                    i++;
                }
            };

            return {
                lines,
                get cursorRow() {
                    return row;
                },
                get cursorCol() {
                    return col;
                },
                write,
            };
        };

        const cols = 80;
        const sim = createTerminalSimulator(cols);

        // Seed terminal with existing history output above the active prompt
        sim.write('Eldritch v0.4.1\r\nConnected to Tavern.\r\n>>> !whoami\r\nroot\r\n>>> ');
        expect(sim.lines.slice(0, 4)).toEqual([
            'Eldritch v0.4.1',
            'Connected to Tavern.',
            '>>> !whoami',
            'root',
        ]);

        let lastCursorRow = 0;
        const redrawBuffer = (buffer: string, cursorPos: number) => {
            const formatted = formatMultilineTerminalBuffer(buffer, cursorPos, '>>> ', '... ');
            const redraw = buildTerminalRedrawSequence(
                formatted.contentToWrite,
                formatted.contentToDisplay,
                formatted.cursorIndex,
                cols,
                lastCursorRow
            );
            sim.write(redraw.sequence);
            lastCursorRow = redraw.cursorRow;
            return redraw;
        };

        // 1. User enters a 4-line function
        let buffer = 'def calc(x):\n    y = x + 1\n    z = y * 2\n    return z';
        let cursorPos = buffer.length;
        redrawBuffer(buffer, cursorPos);

        expect(sim.lines).toEqual([
            'Eldritch v0.4.1',
            'Connected to Tavern.',
            '>>> !whoami',
            'root',
            '>>> def calc(x):',
            '...     y = x + 1',
            '...     z = y * 2',
            '...     return z',
        ]);
        expect(lastCursorRow).toBe(3);

        // 2. User presses Up Arrow 3 times to reach the top line (line 0)
        cursorPos = moveCursorUpInBuffer(buffer, cursorPos)!; // line 2
        redrawBuffer(buffer, cursorPos);
        expect(lastCursorRow).toBe(2);

        cursorPos = moveCursorUpInBuffer(buffer, cursorPos)!; // line 1
        redrawBuffer(buffer, cursorPos);
        expect(lastCursorRow).toBe(1);

        cursorPos = moveCursorUpInBuffer(buffer, cursorPos)!; // line 0
        redrawBuffer(buffer, cursorPos);
        expect(lastCursorRow).toBe(0);

        // History lines above prompt must NOT be overwritten
        expect(sim.lines.slice(0, 4)).toEqual([
            'Eldritch v0.4.1',
            'Connected to Tavern.',
            '>>> !whoami',
            'root',
        ]);

        // 3. User edits line 0 ("def calc(x):" -> "def compute(x):") and line 1 ("y = x + 10")
        const line0EditPos = 'def calc'.length;
        buffer = 'def compute' + buffer.slice(line0EditPos);
        cursorPos = 'def compute'.length;
        redrawBuffer(buffer, cursorPos);

        cursorPos = moveCursorDownInBuffer(buffer, cursorPos)!; // down to line 1
        cursorPos = moveCursorToLineEnd(buffer, cursorPos);
        buffer = buffer.slice(0, cursorPos) + '0' + buffer.slice(cursorPos);
        cursorPos += 1;
        redrawBuffer(buffer, cursorPos);

        // 4. User moves back up to line 0 and then recalls a single-line history item
        cursorPos = moveCursorUpInBuffer(buffer, cursorPos)!;
        redrawBuffer(buffer, cursorPos);
        expect(sim.lines).toEqual([
            'Eldritch v0.4.1',
            'Connected to Tavern.',
            '>>> !whoami',
            'root',
            '>>> def compute(x):',
            '...     y = x + 10',
            '...     z = y * 2',
            '...     return z',
        ]);

        // Recall single-line history item from line 0
        buffer = '!id';
        cursorPos = buffer.length;
        redrawBuffer(buffer, cursorPos);
        expect(sim.lines).toEqual([
            'Eldritch v0.4.1',
            'Connected to Tavern.',
            '>>> !whoami',
            'root',
            '>>> !id',
        ]);
    });

    it('computes wrapped cursor layout accurately when lines wrap across terminal columns', () => {
        // 20-column terminal where line 0 wraps onto 2 visual rows (length 25) and line 1 is 10 chars
        const contentToWrite = '>>> 123456789012345678901\n... return';
        // Cursor at index 20 of line 0 (start of second wrapped visual row)
        const layoutAtWrap = getTerminalCursorLayout(contentToWrite, 20, 20);
        expect(layoutAtWrap).toEqual({
            totalRows: 2,
            cursorRow: 1,
            cursorCol: 0,
        });

        // Cursor at end of line 1
        const layoutAtEnd = getTerminalCursorLayout(contentToWrite, contentToWrite.length, 20);
        expect(layoutAtEnd).toEqual({
            totalRows: 2,
            cursorRow: 2,
            cursorCol: 10,
        });
    });
});

describe('highlightPythonSyntax', () => {
    // New Color Constants
    const COLOR_STRING = "\x1b[38;2;197;148;124m";
    const COLOR_METHOD = "\x1b[38;2;220;220;175m";
    const COLOR_KEYWORD = "\x1b[38;2;188;137;189m";
    const COLOR_OPERAND = "\x1b[38;2;103;155;209m";
    const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
    const COLOR_PUNCTUATION_2 = "\x1b[38;2;204;118;209m";
    const COLOR_COMMENT = "\x1b[90m";
    const COLOR_NUMBER = "\x1b[33m";
    const RESET = "\x1b[0m";

    it('highlights keywords', () => {
        const input = "def my_func(): return True";
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_KEYWORD}def${RESET}`);
        expect(output).toContain(`${COLOR_KEYWORD}return${RESET}`);
        expect(output).toContain(`${COLOR_OPERAND}True${RESET}`);
    });

    it('highlights strings', () => {
        const input = 'print("hello")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}"hello"${RESET}`);
    });

    it('highlights single quoted strings', () => {
        const input = "print('hello')";
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}'hello'${RESET}`);
    });

    it('highlights numbers', () => {
        const input = "x = 123";
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_NUMBER}123${RESET}`);
    });

    it('highlights comments', () => {
        const input = "x = 1 # comment";
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_COMMENT}# comment${RESET}`);
    });

    it('handles mixed content', () => {
        const input = 'if x == "test": # check';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_KEYWORD}if${RESET}`);
        expect(output).toContain(`${COLOR_STRING}"test"${RESET}`);
        expect(output).toContain(`${COLOR_COMMENT}# check${RESET}`);
    });

    it('does not highlight keywords inside strings', () => {
        const input = 'print("def not a keyword")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}"def not a keyword"${RESET}`);
        expect(output).not.toContain(`${COLOR_KEYWORD}def${RESET}`);
    });

    it('does not highlight comments inside strings', () => {
        const input = 'print("# not a comment")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}"# not a comment"${RESET}`);
        expect(output).not.toContain(`${COLOR_COMMENT}# not a comment${RESET}`);
    });

    it('highlights built-ins and methods', () => {
        const input = 'sys.shell("ls")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_METHOD}sys.shell${RESET}`);
    });

    it('alternates punctuation colors based on nesting depth', () => {
        const input = '[ ( { } ) ]';
        const output = highlightPythonSyntax(input);

        // Depth 0 -> 1: [
        expect(output).toContain(`${COLOR_PUNCTUATION_1}[${RESET}`);
        // Depth 1 -> 2: (
        expect(output).toContain(`${COLOR_PUNCTUATION_2}(${RESET}`);
        // Depth 2 -> 3: {
        expect(output).toContain(`${COLOR_PUNCTUATION_1}{${RESET}`);
        // Depth 3 -> 2: }
        expect(output).toContain(`${COLOR_PUNCTUATION_1}}${RESET}`); // Note: depth decrements *before* color selection in current logic?
        // Let's check logic:
        // if closing: depth = max(0, depth - 1); color = depth % 2 == 0 ? P1 : P2;
        // if opening: depth++ (after printing)

        // [ : initial depth 0. color P1 (0%2=0). depth becomes 1.
        // ( : initial depth 1. color P2 (1%2=1). depth becomes 2.
        // { : initial depth 2. color P1 (2%2=0). depth becomes 3.
        // } : closing. depth 3->2. color P1 (2%2=0).
        // ) : closing. depth 2->1. color P2 (1%2=1).
        // ] : closing. depth 1->0. color P1 (0%2=0).

        const parts = output.split(RESET);
        // Clean empty strings from split if any

        // We can check exact sequence using regex match on output
        // Or simpler, check containment of substrings if they are unique enough or use exact match on short string

        // Re-construct expected output manually
        const expected =
            `${COLOR_PUNCTUATION_1}[${RESET} ` +
            `${COLOR_PUNCTUATION_2}(${RESET} ` +
            `${COLOR_PUNCTUATION_1}{${RESET} ` +
            `${COLOR_PUNCTUATION_1}}${RESET} ` +
            `${COLOR_PUNCTUATION_2})${RESET} ` +
            `${COLOR_PUNCTUATION_1}]${RESET}`;

        expect(output).toBe(expected);
    });

    it('highlights f-strings and variables', () => {
        const input = 'f"hello {name}"';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}f"hello `);
        expect(output).toContain(`${RESET}${COLOR_PUNCTUATION_1}{${RESET}name${COLOR_PUNCTUATION_1}}${RESET}${COLOR_STRING}`);
    });

    it('highlights numbers in f-strings', () => {
        const input = 'f"age is {age + 1}"';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${RESET}${COLOR_PUNCTUATION_1}{${RESET}age + ${COLOR_NUMBER}1${RESET}${COLOR_PUNCTUATION_1}}${RESET}${COLOR_STRING}`);
    });

    it('handles escaped braces in f-strings', () => {
        const input = 'f"{{escaped}}"';
        const output = highlightPythonSyntax(input);
        expect(output).toContain(`${COLOR_STRING}f"{{escaped}}"${RESET}`);
        expect(output).not.toContain(`${COLOR_PUNCTUATION_1}{`);
    });
});

describe('Scrollback-Preserving Terminal Clear', () => {
    it('pushes existing viewport lines above the prompt into scrollback on Ctrl+L instead of erasing with CSI 2 J', () => {
        const seq = buildScrollbackPreservingClearSequence(24, 5, 0);
        // Should never emit \x1b[2J or \x1b[3J
        expect(seq).not.toContain('\x1b[2J');
        expect(seq).not.toContain('\x1b[3J');
        // Should erase current prompt (\r\x1b[J), move to row 24, emit 5 linefeeds, and home cursor (\x1b[H)
        expect(seq).toBe('\r\x1b[J\x1b[24;1H' + '\n'.repeat(5) + '\x1b[H');
    });

    it('accounts for multi-line prompt height when clearing at the top or middle of the viewport', () => {
        // Multi-line prompt spanning 2 extra rows ending at cursorY=4 -> promptStartRow=2
        const seq = buildScrollbackPreservingClearSequence(20, 4, 2);
        expect(seq).toBe('\x1b[2A\r\x1b[J\x1b[20;1H\n\n\x1b[H');

        // Already at row 0 -> no extra blank lines pushed into scrollback
        const topSeq = buildScrollbackPreservingClearSequence(20, 0, 0);
        expect(topSeq).toBe('\r\x1b[J\x1b[H');
    });

    it('transforms ANSI clear-screen and clear-scrollback sequences in output to preserve scrollback', () => {
        const raw = '\x1b[3J\x1b[H\x1b[2Jroot@host:~# ';
        const transformed = transformClearAnsiForScrollback(raw, 24, 6);
        expect(transformed).not.toContain('\x1b[3J');
        expect(transformed).not.toContain('\x1b[2J');
        expect(transformed).toBe('\x1b[24;1H' + '\n'.repeat(6) + '\x1b[H\x1b[Jroot@host:~# ');
    });

    it('counts non-empty viewport lines from the active terminal buffer', () => {
        const lines = ['Eldritch v0.4.1', 'Connected to Tavern.', '>>> !whoami', 'root', '', ''];
        const mockTerm = {
            rows: 6,
            buffer: {
                active: {
                    baseY: 10,
                    cursorY: 4,
                    getLine: (idx: number) => ({
                        translateToString: () => lines[idx - 10] ?? '',
                    }),
                },
            },
        };
        expect(getViewportNonEmptyLineCount(mockTerm)).toBe(4);
    });
});

describe('Shell !command macro handling', () => {
    it('submits single-line !command macros immediately on Enter even with colons, quotes, or brackets', () => {
        for (const cmd of [
            '!ls',
            '!whoami',
            '!ls -la /tmp',
            '!echo "hello:"',
            '!echo foo:',
            "!awk '{print $1}' /etc/passwd",
            '!echo "unclosed quote',
            '!echo (unclosed paren',
        ]) {
            expect(shouldContinueMultilineOnEnter(cmd, cmd.length)).toBe(false);
            expect(hasUnclosedBracketsOrStrings(cmd)).toBe(false);
        }
    });

    it('handles !command macros inside multi-line blocks cleanly', () => {
        const blockWithMacro = 'if True:\n    !echo foo:\n    ';
        expect(hasUnclosedBracketsOrStrings(blockWithMacro)).toBe(false);
        // Pressing Enter on the trailing blank line submits the block
        expect(shouldContinueMultilineOnEnter(blockWithMacro, blockWithMacro.length)).toBe(false);

        // Pressing Enter at the end of an indented `!echo foo:` line preserves indent without adding +4 spaces
        const midBlock = 'if True:\n    !echo foo:';
        const inserted = insertNewlineWithAutoIndent(midBlock, midBlock.length);
        expect(inserted.buffer).toBe('if True:\n    !echo foo:\n    ');
    });
});
