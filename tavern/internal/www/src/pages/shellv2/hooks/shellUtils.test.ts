import { describe, it, expect } from 'vitest';
import { moveWordLeft, moveWordRight, highlightPythonSyntax } from './shellUtils';

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
    it('highlights keywords', () => {
        const input = "def my_func(): return True";
        const output = highlightPythonSyntax(input);
        expect(output).toContain("\x1b[35mdef\x1b[0m");
        expect(output).toContain("\x1b[35mreturn\x1b[0m");
        expect(output).toContain("\x1b[35mTrue\x1b[0m");
    });

    it('highlights strings', () => {
        const input = 'print("hello")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain('\x1b[32m"hello"\x1b[0m');
    });

    it('highlights single quoted strings', () => {
        const input = "print('hello')";
        const output = highlightPythonSyntax(input);
        expect(output).toContain("\x1b[32m'hello'\x1b[0m");
    });

    it('highlights numbers', () => {
        const input = "x = 123";
        const output = highlightPythonSyntax(input);
        expect(output).toContain("\x1b[33m123\x1b[0m");
    });

    it('highlights comments', () => {
        const input = "x = 1 # comment";
        const output = highlightPythonSyntax(input);
        expect(output).toContain("\x1b[90m# comment\x1b[0m");
    });

    it('handles mixed content', () => {
        const input = 'if x == "test": # check';
        const output = highlightPythonSyntax(input);
        expect(output).toContain("\x1b[35mif\x1b[0m");
        expect(output).toContain('\x1b[32m"test"\x1b[0m');
        expect(output).toContain("\x1b[90m# check\x1b[0m");
    });

    it('does not highlight keywords inside strings', () => {
        const input = 'print("def not a keyword")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain('\x1b[32m"def not a keyword"\x1b[0m');
        expect(output).not.toContain("\x1b[35mdef\x1b[0m");
    });

    it('does not highlight comments inside strings', () => {
        const input = 'print("# not a comment")';
        const output = highlightPythonSyntax(input);
        expect(output).toContain('\x1b[32m"# not a comment"\x1b[0m');
        expect(output).not.toContain("\x1b[90m# not a comment\x1b[0m");
    });
});
