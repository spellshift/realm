import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { moveWordLeft, moveWordRight, highlightPythonSyntax, loadHistory, saveHistory, HISTORY_KEY, MAX_HISTORY } from './shellUtils';

describe('History Persistence', () => {
    beforeEach(() => {
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
});
