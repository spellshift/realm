import { describe, it, expect } from 'vitest';
import { moveWordLeft, moveWordRight } from './shellUtils';

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
