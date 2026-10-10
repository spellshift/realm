import { useEffect } from 'react';

/**
 * Registers ctrl+[0-9] keyboard shortcuts for switching between tabs.
 * ctrl+1 switches to the first tab (index 0), ctrl+2 to the second, etc.
 * ctrl+0 is treated as the 10th tab (index 9).
 */
export function useTabHotkeys(totalTabs: number, setTabIndex: (index: number) => void) {
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (!e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) return;

            const digit = parseInt(e.key, 10);
            if (isNaN(digit) || digit < 0 || digit > 9) return;

            // ctrl+1 => index 0, ctrl+2 => index 1, ..., ctrl+0 => index 9
            const targetIndex = digit === 0 ? 9 : digit - 1;
            if (targetIndex < totalTabs) {
                e.preventDefault();
                setTabIndex(targetIndex);
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [totalTabs, setTabIndex]);
}
