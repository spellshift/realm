import { useState, useRef, useCallback, RefObject } from "react";
import { Terminal } from "@xterm/xterm";

export interface CompletionsState {
    list: string[];
    start: number;
    show: boolean;
    index: number;
}

export const useTerminalCompletions = (termInstance: RefObject<Terminal | null>) => {
    const [completions, setCompletions] = useState<string[]>([]);
    const [completionStart, setCompletionStart] = useState(0);
    const [showCompletions, setShowCompletions] = useState(false);
    const [completionIndex, setCompletionIndex] = useState(0);
    const [completionPos, setCompletionPos] = useState({ x: 0, y: 0 });

    const completionsRef = useRef<CompletionsState>({
        list: [], start: 0, show: false, index: 0
    });

    const updateCompletionsUI = useCallback((list: string[], start: number, show: boolean, index: number) => {
        setCompletions(list);
        setCompletionStart(start);
        setShowCompletions(show);
        setCompletionIndex(index);
        completionsRef.current = { list, start, show, index };

        if (show && termInstance.current) {
            const cursorX = termInstance.current.buffer.active.cursorX;
            const cursorY = termInstance.current.buffer.active.cursorY;
            const charWidth = 9;
            const charHeight = 17;
            setCompletionPos({
                x: cursorX * charWidth + 20,
                y: cursorY * charHeight + 40
            });
        }
    }, [termInstance]);

    return {
        completions,
        completionStart,
        showCompletions,
        completionIndex,
        completionPos,
        updateCompletionsUI,
        completionsRef
    };
};
