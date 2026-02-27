import { useState, useRef, useCallback, RefObject } from "react";
import { Terminal } from "@xterm/xterm";
import docsData from "../../../assets/eldritch-docs.json";

const docs = docsData as Record<string, { signature: string; description: string }>;

interface TooltipState {
    visible: boolean;
    x: number;
    y: number;
    signature: string;
    description: string;
}

export const useTerminalTooltip = (
    termInstance: RefObject<Terminal | null>,
    termRef: RefObject<HTMLDivElement>
) => {
    const [tooltipState, setTooltipState] = useState<TooltipState>({
        visible: false,
        x: 0,
        y: 0,
        signature: "",
        description: ""
    });

    const currentTooltipWord = useRef<string | null>(null);

    const handleMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
        if (!termInstance.current || !termRef.current) return;

        const rect = termRef.current.getBoundingClientRect();
        const cols = termInstance.current.cols;
        const rows = termInstance.current.rows;

        const cellWidth = rect.width / cols;
        const cellHeight = rect.height / rows;

        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        const col = Math.floor(x / cellWidth);
        const row = Math.floor(y / cellHeight);

        if (col < 0 || col >= cols || row < 0 || row >= rows) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        const buffer = termInstance.current.buffer.active;
        const bufferRowIndex = buffer.viewportY + row;
        const line = buffer.getLine(bufferRowIndex);

        if (!line) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        const lineStr = line.translateToString(true);

        if (col >= lineStr.length) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        const allowedChars = /[a-zA-Z0-9_\.]/;
        if (!allowedChars.test(lineStr[col])) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        let start = col;
        while (start > 0 && allowedChars.test(lineStr[start - 1])) {
            start--;
        }

        let end = col;
        while (end < lineStr.length && allowedChars.test(lineStr[end])) {
            end++;
        }

        const word = lineStr.slice(start, end);

        if (docs[word]) {
            if (currentTooltipWord.current === word && tooltipState.visible) {
                return;
            }

            currentTooltipWord.current = word;
            setTooltipState({
                visible: true,
                x: e.clientX,
                y: e.clientY,
                signature: docs[word].signature,
                description: docs[word].description
            });
        } else {
             currentTooltipWord.current = null;
             setTooltipState(s => {
                 if (!s.visible) return s;
                 return { ...s, visible: false };
             });
        }
    }, [tooltipState.visible, termInstance, termRef]);

    return {
        tooltipState,
        handleMouseMove
    };
};
