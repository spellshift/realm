import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { PollingProvider, usePolling } from '../PollingContext';

// Mock Apollo Client
const mockRefetchQueries = vi.fn();
vi.mock('@apollo/client', () => ({
    useApolloClient: () => ({
        refetchQueries: mockRefetchQueries,
    }),
}));

describe('PollingContext', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('PollingProvider initialization', () => {
        it('should provide default seconds until next poll', () => {
            const { result } = renderHook(() => usePolling(), {
                wrapper: PollingProvider,
            });

            expect(result.current.secondsUntilNextPoll).toBe(30);
        });

        it('should throw error when usePolling is used outside PollingProvider', () => {
            const consoleError = vi.spyOn(console, 'error').mockImplementation(() => { });

            try {
                renderHook(() => usePolling());
                expect(false).toBe(true);
            } catch (error: any) {
                expect(error.message).toContain('usePolling must be used within a PollingContextProvider');
            }

            consoleError.mockRestore();
        });
    });

    describe('Provider setup', () => {
        it('should set up intervals on mount', () => {
            const setIntervalSpy = vi.spyOn(global, 'setInterval');

            renderHook(() => usePolling(), {
                wrapper: PollingProvider,
            });

            // Should set up two intervals: one for polling, one for countdown
            expect(setIntervalSpy).toHaveBeenCalledTimes(2);
            expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 30000);
            expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 1000);

            setIntervalSpy.mockRestore();
        });

        it('should clean up intervals on unmount', () => {
            const clearIntervalSpy = vi.spyOn(global, 'clearInterval');

            const { unmount } = renderHook(() => usePolling(), {
                wrapper: PollingProvider,
            });

            unmount();

            expect(clearIntervalSpy).toHaveBeenCalledTimes(2);

            clearIntervalSpy.mockRestore();
        });
    });
});
