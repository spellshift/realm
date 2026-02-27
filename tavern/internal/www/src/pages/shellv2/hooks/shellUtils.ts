export const HISTORY_KEY = "eldritch_shell_history";
export const MAX_HISTORY = 1000;

export const loadHistory = (): string[] => {
    try {
        const stored = localStorage.getItem(HISTORY_KEY);
        if (stored) {
            const parsed = JSON.parse(stored);
            if (Array.isArray(parsed)) {
                return parsed;
            }
        }
    } catch (e) {
        console.error("Failed to load history", e);
    }
    return [];
};

export const saveHistory = (history: string[]) => {
    try {
        const sliced = history.slice(-MAX_HISTORY);
        localStorage.setItem(HISTORY_KEY, JSON.stringify(sliced));
    } catch (e) {
        console.error("Failed to save history", e);
    }
};

export const moveWordLeft = (buffer: string, cursor: number): number => {
    const beforeCursor = buffer.slice(0, cursor);
    const trimmed = beforeCursor.trimEnd();
    const lastSpace = trimmed.lastIndexOf(" ");
    return lastSpace === -1 ? 0 : lastSpace + 1;
};

export const moveWordRight = (buffer: string, cursor: number): number => {
    const len = buffer.length;
    let i = cursor;

    // 1. If we are at the end, stay there
    if (i >= len) return len;

    // 2. If we are on a space, skip all spaces first
    while (i < len && buffer[i] === ' ') {
        i++;
    }

    // 3. Now skip all non-spaces (the word)
    while (i < len && buffer[i] !== ' ') {
        i++;
    }

    return i;
};

export const highlightPythonSyntax = (input: string): string => {
    const regex = /((["'])(?:\\.|[^\\])*?\2)|(#.*)|(\b(?:def|class|import|from|return|if|else|elif|while|for|in|try|except|finally|with|as|pass|break|continue|lambda|yield|global|nonlocal|assert|del|raise|True|False|None|and|or|not|is|print|exec|eval|open|range|len)\b)|(\b\d+\b)/g;

    let lastIndex = 0;
    let result = "";
    let match;

    while ((match = regex.exec(input)) !== null) {
        // Add plain text before match
        if (match.index > lastIndex) {
            result += input.slice(lastIndex, match.index);
        }

        const text = match[0];
        if (match[1]) {
            // String (Green)
            result += `\x1b[32m${text}\x1b[0m`;
        } else if (match[3]) {
            // Comment (Grey)
            result += `\x1b[90m${text}\x1b[0m`;
        } else if (match[4]) {
            // Keyword (Magenta)
            result += `\x1b[35m${text}\x1b[0m`;
        } else if (match[5]) {
            // Number (Yellow)
            result += `\x1b[33m${text}\x1b[0m`;
        } else {
            result += text;
        }

        lastIndex = regex.lastIndex;
    }

    // Add remaining text
    if (lastIndex < input.length) {
        result += input.slice(lastIndex);
    }

    return result;
};
