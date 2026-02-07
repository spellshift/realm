import { describe, it, expect } from 'vitest';
import { formatRelativeTime } from './time';
import { subSeconds, subMinutes, subHours, subDays } from 'date-fns';

describe('formatRelativeTime', () => {
    const now = new Date();

    it('formats seconds correctly', () => {
        const date = subSeconds(now, 5);
        expect(formatRelativeTime(date)).toBe('5s ago');
    });

    it('formats minutes correctly', () => {
        const date = subMinutes(now, 15);
        // Note: subMinutes(15) is exactly 900 seconds.
        // My implementation: 900 / 60 = 15m.
        expect(formatRelativeTime(date)).toBe('15m ago');
    });

    it('formats hours and minutes correctly', () => {
        const date = subMinutes(subHours(now, 1), 15);
        // 1h 15m ago
        expect(formatRelativeTime(date)).toBe('1h 15m ago');
    });

    it('formats hours correctly without minutes', () => {
        const date = subHours(now, 2);
        // 2h ago
        expect(formatRelativeTime(date)).toBe('2h ago');
    });

    it('formats >1d correctly', () => {
        const date = subDays(now, 2);
        expect(formatRelativeTime(date)).toBe('>1d ago');
    });
});
