import docsData from "../../../assets/eldritch-docs.json";

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

const COLOR_STRING = "\x1b[38;2;197;148;124m";
const COLOR_METHOD = "\x1b[38;2;220;220;175m";
const COLOR_KEYWORD = "\x1b[38;2;188;137;189m";
const COLOR_OPERAND = "\x1b[38;2;103;155;209m";
const COLOR_PUNCTUATION_1 = "\x1b[38;2;249;217;73m";
const COLOR_PUNCTUATION_2 = "\x1b[38;2;204;118;209m";
const COLOR_COMMENT = "\x1b[90m";
const COLOR_NUMBER = "\x1b[33m";
const RESET = "\x1b[0m";

export const highlightPythonSyntax = (input: string): string => {
    const builtins = Object.keys(docsData).map(k => k.replace(/\./g, "\\."));
    const builtinsPattern = builtins.join("|");

    // Order matters for regex capture groups
    const regex = new RegExp([
        // 1. Strings (single/double quoted)
        /((["'])(?:\\.|[^\\])*?\2)/.source,
        // 2. Comments (#...)
        /(#.*)/.source,
        // 3. Keywords
        /(\b(?:def|class|import|from|return|if|else|elif|while|for|try|except|finally|with|as|pass|break|continue|lambda|yield|global|nonlocal|assert|del|raise)\b)/.source,
        // 4. Operands
        /(\b(?:is|not|in|and|or|True|False|None)\b)/.source,
        // 5. Built-ins / Methods
        `(\\b(?:${builtinsPattern})\\b)`,
        // 6. Numbers
        /(\b\d+\b)/.source,
        // 7. Punctuation
        /([()[\]{}])/.source
    ].join("|"), "g");

    let lastIndex = 0;
    let result = "";
    let match;
    let depth = 0;

    while ((match = regex.exec(input)) !== null) {
        // Add plain text before match
        if (match.index > lastIndex) {
            result += input.slice(lastIndex, match.index);
        }

        const text = match[0];
        if (match[1]) {
            // String
            result += `${COLOR_STRING}${text}${RESET}`;
        } else if (match[3]) {
            // Comment
            result += `${COLOR_COMMENT}${text}${RESET}`;
        } else if (match[4]) {
            // Keyword
            result += `${COLOR_KEYWORD}${text}${RESET}`;
        } else if (match[5]) {
            // Operand
            result += `${COLOR_OPERAND}${text}${RESET}`;
        } else if (match[6]) {
            // Built-in / Method
            result += `${COLOR_METHOD}${text}${RESET}`;
        } else if (match[7]) {
            // Number
            result += `${COLOR_NUMBER}${text}${RESET}`;
        } else if (match[8]) {
            // Punctuation
            if (/[)\]}]/.test(text)) {
                depth = Math.max(0, depth - 1);
            }

            const color = depth % 2 === 0 ? COLOR_PUNCTUATION_1 : COLOR_PUNCTUATION_2;
            result += `${color}${text}${RESET}`;

            if (/[[({]/.test(text)) {
                depth++;
            }
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
