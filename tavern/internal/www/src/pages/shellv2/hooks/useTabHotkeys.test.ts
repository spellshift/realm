import { renderHook } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { useTabHotkeys } from './useTabHotkeys';

function fireCtrlKey(key: string, extras: Partial<KeyboardEventInit> = {}) {
    const event = new KeyboardEvent('keydown', {
        key,
        ctrlKey: true,
        bubbles: true,
        cancelable: true,
        ...extras,
    });
    const spy = vi.spyOn(event, 'preventDefault');
    window.dispatchEvent(event);
    return spy;
}

describe('useTabHotkeys', () => {
    it('ctrl+1 switches to the first tab (index 0)', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('1');
        expect(setTabIndex).toHaveBeenCalledWith(0);
    });

    it('ctrl+2 switches to the second tab (index 1)', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('2');
        expect(setTabIndex).toHaveBeenCalledWith(1);
    });

    it('ctrl+0 is treated as 10th tab (index 9)', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(10, setTabIndex));

        fireCtrlKey('0');
        expect(setTabIndex).toHaveBeenCalledWith(9);
    });

    it('does not switch if target tab index exceeds total tabs', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(2, setTabIndex));

        fireCtrlKey('5');
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('ctrl+0 does not switch when fewer than 10 tabs', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('0');
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('ignores non-digit keys', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('a');
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('ignores when alt is also pressed', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('1', { altKey: true });
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('ignores when shift is also pressed', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('1', { shiftKey: true });
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('ignores when meta is also pressed', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        fireCtrlKey('1', { metaKey: true });
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('calls preventDefault on valid hotkey', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(3, setTabIndex));

        const spy = fireCtrlKey('1');
        expect(spy).toHaveBeenCalled();
    });

    it('does not call preventDefault on invalid hotkey', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(2, setTabIndex));

        const spy = fireCtrlKey('5');
        expect(spy).not.toHaveBeenCalled();
    });

    it('cleans up event listener on unmount', () => {
        const setTabIndex = vi.fn();
        const { unmount } = renderHook(() => useTabHotkeys(3, setTabIndex));

        unmount();
        fireCtrlKey('1');
        expect(setTabIndex).not.toHaveBeenCalled();
    });

    it('handles all digits 1-9 correctly', () => {
        const setTabIndex = vi.fn();
        renderHook(() => useTabHotkeys(10, setTabIndex));

        for (let i = 1; i <= 9; i++) {
            setTabIndex.mockClear();
            fireCtrlKey(String(i));
            expect(setTabIndex).toHaveBeenCalledWith(i - 1);
        }
    });
});
