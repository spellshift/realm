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

export const stripInternalMacro = (input: string): string => {
    const prefix = "for _nonomacroclowntown in range(1):\n\t_nonomacroclowntown = ";
    const suffix = "print(_nonomacroclowntown['stdout']);print(_nonomacroclowntown['stderr'])\n";
    let cleaned = input;
    if (cleaned.startsWith(prefix)) {
        cleaned = cleaned.slice(prefix.length);
    }
    if (cleaned.endsWith(suffix)) {
        cleaned = cleaned.slice(0, -suffix.length);
    }
    return cleaned.trim();
};

/**
 * Counts how many rows from the top of the current viewport contain non-empty text.
 */
export const getViewportNonEmptyLineCount = (term: {
    rows?: number;
    buffer?: {
        active?: {
            baseY: number;
            cursorY: number;
            getLine: (y: number) => { translateToString: (trimRight?: boolean) => string } | undefined;
        };
    };
}): number => {
    const rows = Math.max(1, term.rows || 24);
    const active = term.buffer?.active;
    if (!active) return 0;

    for (let r = rows - 1; r >= 0; r--) {
        const line = active.getLine(active.baseY + r);
        if (line && line.translateToString(true).length > 0) {
            return r + 1;
        }
    }
    return Math.max(0, active.cursorY || 0);
};

/**
 * Builds an ANSI escape sequence that clears the visible viewport while pushing
 * all lines above the current prompt into the terminal's scrollback buffer,
 * so scrolling up preserves the complete session input/output history.
 */
export const buildScrollbackPreservingClearSequence = (
    rows: number,
    cursorY: number,
    prevPromptRows: number = 0
): string => {
    const safeRows = Math.max(1, rows || 24);
    const safePrevRows = Math.max(0, prevPromptRows);
    const promptStartRow = Math.max(0, (cursorY || 0) - safePrevRows);

    let seq = "";
    if (safePrevRows > 0) {
        seq += `\x1b[${safePrevRows}A`;
    }
    // Erase the current unsubmitted prompt line(s) so they aren't duplicated in scrollback
    seq += "\r\x1b[J";

    if (promptStartRow > 0) {
        // Move to the bottom row of the viewport and emit `promptStartRow` linefeeds
        // so rows 0 .. promptStartRow - 1 scroll up into the scrollback buffer.
        seq += `\x1b[${safeRows};1H` + "\n".repeat(promptStartRow);
    }

    // Home cursor to top-left of the clean viewport
    seq += "\x1b[H";
    return seq;
};

/**
 * Transforms ANSI clear-screen / clear-scrollback sequences (\x1b[2J, \x1b[3J)
 * in terminal output so existing viewport lines are scrolled into scrollback
 * instead of being erased in-place.
 */
export const transformClearAnsiForScrollback = (
    data: string,
    rows: number,
    viewportNonEmptyLines: number
): string => {
    if (!data || !data.includes("\x1b[")) return data;

    // Never allow CSI 3 J to wipe the scrollback buffer
    let result = data.replace(/\x1b\[3J/g, "");

    if (!/\x1b\[2J/.test(result)) {
        return result;
    }

    const safeRows = Math.max(1, rows || 24);
    const linesToScroll = Math.max(0, Math.min(safeRows, viewportNonEmptyLines));
    const scrollSeq =
        linesToScroll > 0
            ? `\x1b[${safeRows};1H` + "\n".repeat(linesToScroll) + "\x1b[H\x1b[J"
            : "\x1b[H\x1b[J";

    result = result.replace(/\x1b\[(?:H|\d*;\d*H)?\x1b\[2J(?:\x1b\[(?:H|\d*;\d*H))?/g, scrollSeq);
    return result;
};

export const moveWordLeft = (buffer: string, cursor: number): number => {
    const beforeCursor = buffer.slice(0, cursor);
    const trimmed = beforeCursor.replace(/\s+$/, "");
    const lastSpace = Math.max(
        trimmed.lastIndexOf(" "),
        trimmed.lastIndexOf("\n"),
        trimmed.lastIndexOf("\t")
    );
    return lastSpace === -1 ? 0 : lastSpace + 1;
};

export const moveWordRight = (buffer: string, cursor: number): number => {
    const len = buffer.length;
    let i = cursor;

    // 1. If we are at the end, stay there
    if (i >= len) return len;

    // 2. If we are on whitespace, skip all whitespace first
    while (i < len && /\s/.test(buffer[i])) {
        i++;
    }

    // 3. Now skip all non-whitespace (the word)
    while (i < len && !/\s/.test(buffer[i])) {
        i++;
    }

    return i;
};

export const isInsideString = (buffer: string, cursor: number): boolean => {
    let insideSingle = false;
    let insideDouble = false;
    let escape = false;

    for (let i = 0; i < cursor; i++) {
        const char = buffer[i];

        if (escape) {
            escape = false;
        } else if (char === '\\') {
            escape = true;
        } else if (char === '\n') {
            escape = false;
            insideSingle = false;
            insideDouble = false;
        } else if (char === "'") {
            if (!insideDouble) {
                insideSingle = !insideSingle;
            }
        } else if (char === '"') {
            if (!insideSingle) {
                insideDouble = !insideDouble;
            }
        }
    }

    return insideSingle || insideDouble;
};

export interface CursorLineInfo {
    lines: string[];
    lineIndex: number;
    col: number;
    lineStartOffset: number;
    lineEndOffset: number;
}

export const getCursorLineInfo = (buffer: string, cursorPos: number): CursorLineInfo => {
    const clampedCursor = Math.max(0, Math.min(cursorPos, buffer.length));
    const lines = buffer.split("\n");
    let offset = 0;

    for (let i = 0; i < lines.length; i++) {
        const lineLen = lines[i].length;
        const lineStart = offset;
        const lineEnd = offset + lineLen;
        if (clampedCursor <= lineEnd || i === lines.length - 1) {
            return {
                lines,
                lineIndex: i,
                col: clampedCursor - lineStart,
                lineStartOffset: lineStart,
                lineEndOffset: lineEnd,
            };
        }
        offset = lineEnd + 1; // +1 for '\n'
    }

    return {
        lines,
        lineIndex: 0,
        col: 0,
        lineStartOffset: 0,
        lineEndOffset: lines[0]?.length ?? 0,
    };
};

export const moveCursorUpInBuffer = (
    buffer: string,
    cursorPos: number,
    preferredCol?: number | null
): number | null => {
    const { lines, lineIndex, col } = getCursorLineInfo(buffer, cursorPos);
    if (lineIndex <= 0) {
        return null;
    }
    let targetLineStart = 0;
    for (let i = 0; i < lineIndex - 1; i++) {
        targetLineStart += lines[i].length + 1;
    }
    const desiredCol = typeof preferredCol === "number" ? preferredCol : col;
    const targetCol = Math.min(desiredCol, lines[lineIndex - 1].length);
    return targetLineStart + targetCol;
};

export const moveCursorDownInBuffer = (
    buffer: string,
    cursorPos: number,
    preferredCol?: number | null
): number | null => {
    const { lines, lineIndex, col } = getCursorLineInfo(buffer, cursorPos);
    if (lineIndex >= lines.length - 1) {
        return null;
    }
    let targetLineStart = 0;
    for (let i = 0; i <= lineIndex; i++) {
        targetLineStart += lines[i].length + 1;
    }
    const desiredCol = typeof preferredCol === "number" ? preferredCol : col;
    const targetCol = Math.min(desiredCol, lines[lineIndex + 1].length);
    return targetLineStart + targetCol;
};

export const moveCursorToLineStart = (buffer: string, cursorPos: number): number => {
    return getCursorLineInfo(buffer, cursorPos).lineStartOffset;
};

export const moveCursorToLineEnd = (buffer: string, cursorPos: number): number => {
    return getCursorLineInfo(buffer, cursorPos).lineEndOffset;
};

export const dedentCurrentLine = (
    buffer: string,
    cursorPos: number
): { buffer: string; cursorPos: number } => {
    const { lines, lineIndex, lineStartOffset } = getCursorLineInfo(buffer, cursorPos);
    const currentLine = lines[lineIndex] ?? "";
    const match = currentLine.match(/^( {1,4})/);
    if (!match) {
        return { buffer, cursorPos };
    }
    const removeCount = match[1].length;
    const nextBuffer =
        buffer.slice(0, lineStartOffset) + buffer.slice(lineStartOffset + removeCount);
    const nextCursorPos = Math.max(lineStartOffset, cursorPos - removeCount);
    return { buffer: nextBuffer, cursorPos: nextCursorPos };
};

export const stripTrailingCommentOutsideStrings = (line: string): string => {
    let inSingle = false;
    let inDouble = false;
    let escape = false;
    for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (escape) {
            escape = false;
            continue;
        }
        if (ch === "\\") {
            escape = true;
            continue;
        }
        if (ch === "'" && !inDouble) {
            inSingle = !inSingle;
        } else if (ch === '"' && !inSingle) {
            inDouble = !inDouble;
        } else if (ch === "#" && !inSingle && !inDouble) {
            return line.slice(0, i);
        }
    }
    return line;
};

export const hasUnclosedBracketsOrStrings = (buffer: string): boolean => {
    let balance = 0;
    let inSingle = false;
    let inDouble = false;
    let inTripleSingle = false;
    let inTripleDouble = false;
    let inComment = false;
    let escape = false;

    for (let i = 0; i < buffer.length; i++) {
        // At the start of any line outside multi-line triple-quoted strings,
        // if the line is a `!command` shell macro, skip to the end of the line
        // because `expand_macros` escapes the entire line into `sys.shell("...")`.
        if (
            (i === 0 || buffer[i - 1] === "\n") &&
            !inTripleSingle &&
            !inTripleDouble &&
            !inSingle &&
            !inDouble
        ) {
            const lineEnd = buffer.indexOf("\n", i);
            const lineSlice = lineEnd === -1 ? buffer.slice(i) : buffer.slice(i, lineEnd);
            if (lineSlice.trimStart().startsWith("!")) {
                if (lineEnd === -1) {
                    break;
                }
                i = lineEnd; // Loop's i++ will advance to the start of the next line
                continue;
            }
        }

        const ch = buffer[i];
        const nextTwo = buffer.slice(i, i + 3);

        if (inComment) {
            if (ch === "\n") {
                inComment = false;
            }
            continue;
        }

        if (escape) {
            escape = false;
            continue;
        }

        if (ch === "\\") {
            escape = true;
            continue;
        }

        if (inTripleSingle) {
            if (nextTwo === "'''") {
                inTripleSingle = false;
                i += 2;
            }
            continue;
        }

        if (inTripleDouble) {
            if (nextTwo === '"""') {
                inTripleDouble = false;
                i += 2;
            }
            continue;
        }

        if (inSingle) {
            if (ch === "\n") {
                return true; // Unterminated single-quoted string on a line
            }
            if (ch === "'") {
                inSingle = false;
            }
            continue;
        }

        if (inDouble) {
            if (ch === "\n") {
                return true; // Unterminated double-quoted string on a line
            }
            if (ch === '"') {
                inDouble = false;
            }
            continue;
        }

        if (nextTwo === "'''") {
            inTripleSingle = true;
            i += 2;
            continue;
        }

        if (nextTwo === '"""') {
            inTripleDouble = true;
            i += 2;
            continue;
        }

        if (ch === "'") {
            inSingle = true;
            continue;
        }

        if (ch === '"') {
            inDouble = true;
            continue;
        }

        if (ch === "#") {
            inComment = true;
            continue;
        }

        if (ch === "(" || ch === "[" || ch === "{") {
            balance++;
        } else if (ch === ")" || ch === "]" || ch === "}") {
            if (balance > 0) {
                balance--;
            }
        }
    }

    if (escape || inSingle || inDouble || inTripleSingle || inTripleDouble || balance > 0) {
        return true;
    }

    return false;
};

export const shouldContinueMultilineOnEnter = (buffer: string, cursorPos: number): boolean => {
    const { lines, lineIndex } = getCursorLineInfo(buffer, cursorPos);

    // Single-line `!command` shell macros always execute immediately on Enter
    if (lines.length === 1 && lines[0].trimStart().startsWith("!")) {
        return false;
    }

    // If the cursor is on an earlier line of a multi-line buffer, Enter splits/adds a line in place
    if (lineIndex < lines.length - 1) {
        return true;
    }

    // If brackets or quotes are unclosed anywhere in the buffer, keep editing in the buffer
    if (hasUnclosedBracketsOrStrings(buffer)) {
        return true;
    }

    if (lines.length === 1) {
        const stripped = stripTrailingCommentOutsideStrings(lines[0]).trimEnd();
        return stripped.endsWith(":") || stripped.endsWith("\\");
    }

    // Multi-line buffer (lines.length > 1) where cursor is on the last line:
    // Continue adding lines until the user presses Enter on an empty/whitespace-only trailing line.
    const lastLine = lines[lines.length - 1];
    if (lastLine.trim() !== "") {
        return true;
    }

    // Last line is empty/whitespace-only; verify the previous non-empty line wasn't a bare block header ':'
    for (let i = lines.length - 2; i >= 0; i--) {
        if (lines[i].trim() !== "") {
            if (!lines[i].trimStart().startsWith("!")) {
                const prevStripped = stripTrailingCommentOutsideStrings(lines[i]).trimEnd();
                if (prevStripped.endsWith(":") || prevStripped.endsWith("\\")) {
                    return true;
                }
            }
            break;
        }
    }

    return false;
};

export const insertNewlineWithAutoIndent = (
    buffer: string,
    cursorPos: number
): { buffer: string; cursorPos: number } => {
    const { lines, lineIndex, col } = getCursorLineInfo(buffer, cursorPos);
    const currentLine = lines[lineIndex] ?? "";
    const beforeCursorOnLine = currentLine.slice(0, col);
    const indentMatch = currentLine.match(/^(\s*)/);
    let indent = indentMatch ? indentMatch[1] : "";

    const isMacroLine = currentLine.trimStart().startsWith("!");
    const strippedBefore = stripTrailingCommentOutsideStrings(beforeCursorOnLine).trimEnd();
    if (!isMacroLine && strippedBefore.endsWith(":")) {
        indent += "    ";
    } else if (!isMacroLine && /^\s*(?:return|break|continue|pass|raise)\b/.test(beforeCursorOnLine)) {
        indent = indent.length >= 4 ? indent.slice(0, indent.length - 4) : "";
    }

    const insertion = "\n" + indent;
    const nextBuffer = buffer.slice(0, cursorPos) + insertion + buffer.slice(cursorPos);
    return {
        buffer: nextBuffer,
        cursorPos: cursorPos + insertion.length,
    };
};

export const formatMultilineTerminalBuffer = (
    buffer: string,
    cursorPos: number,
    basePrompt = ">>> ",
    contPrompt = "... "
): { contentToWrite: string; contentToDisplay: string; cursorIndex: number } => {
    const lines = buffer.split("\n");
    const plainLines = lines.map((line, idx) => (idx === 0 ? basePrompt : contPrompt) + line);
    const displayLines = lines.map(
        (line, idx) =>
            (idx === 0 ? basePrompt : `\x1b[90m${contPrompt}\x1b[0m`) + highlightPythonSyntax(line)
    );

    const clampedCursor = Math.max(0, Math.min(cursorPos, buffer.length));
    const lineIndex = buffer.slice(0, clampedCursor).split("\n").length - 1;
    const cursorIndex = clampedCursor + basePrompt.length + lineIndex * contPrompt.length;

    return {
        contentToWrite: plainLines.join("\n"),
        contentToDisplay: displayLines.join("\n"),
        cursorIndex,
    };
};

export const getVisualLineCount = (text: string, cols: number): number => {
    const safeCols = Math.max(1, cols || 80);
    const lines = text.split("\n");
    let count = 0;
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (i > 0) count++;
        if (line.length > 0) {
            count += Math.floor((line.length - 1) / safeCols);
        }
    }
    return count;
};

export interface TerminalCursorLayout {
    totalRows: number;
    cursorRow: number;
    cursorCol: number;
}

export const getTerminalCursorLayout = (
    contentToWrite: string,
    cursorIndex: number,
    cols: number
): TerminalCursorLayout => {
    const safeCols = Math.max(1, cols || 80);
    const clampedCursor = Math.max(0, Math.min(cursorIndex, contentToWrite.length));
    const fullLines = contentToWrite.split("\n");
    const prefix = contentToWrite.slice(0, clampedCursor);
    const prefixLines = prefix.split("\n");
    const cursorLineIdx = prefixLines.length - 1;
    const beforeCursorOnLine = prefixLines[cursorLineIdx] ?? "";

    let totalRows = 0;
    let cursorRow = 0;
    let cursorCol = 0;

    for (let i = 0; i < fullLines.length; i++) {
        const lineLen = fullLines[i].length;
        const lineExtraRows = lineLen > 0 ? Math.floor((lineLen - 1) / safeCols) : 0;

        if (i > 0) {
            totalRows += 1;
        }
        totalRows += lineExtraRows;

        if (i < cursorLineIdx) {
            cursorRow += 1 + lineExtraRows;
        } else if (i === cursorLineIdx) {
            const b = beforeCursorOnLine.length;
            if (b === 0) {
                cursorCol = 0;
            } else if (b < lineLen) {
                cursorRow += Math.floor(b / safeCols);
                cursorCol = b % safeCols;
            } else {
                // b === lineLen and lineLen > 0
                if (lineLen % safeCols === 0) {
                    cursorRow += lineLen / safeCols - 1;
                    cursorCol = safeCols - 1;
                } else {
                    cursorRow += Math.floor(lineLen / safeCols);
                    cursorCol = lineLen % safeCols;
                }
            }
        }
    }

    return {
        totalRows,
        cursorRow,
        cursorCol,
    };
};

export interface TerminalRedrawResult {
    sequence: string;
    totalRows: number;
    cursorRow: number;
    cursorCol: number;
}

export const buildTerminalRedrawSequence = (
    contentToWrite: string,
    contentToDisplay: string,
    cursorIndex: number,
    cols: number,
    prevCursorRow: number
): TerminalRedrawResult => {
    const { totalRows, cursorRow, cursorCol } = getTerminalCursorLayout(
        contentToWrite,
        cursorIndex,
        cols
    );

    let seq = "";
    const safePrevCursorRow = Math.max(0, prevCursorRow);
    if (safePrevCursorRow > 0) {
        seq += `\x1b[${safePrevCursorRow}A`;
    }

    seq += "\r\x1b[J";
    seq += contentToDisplay.replace(/\n/g, "\r\n");

    const moveUp = totalRows - cursorRow;
    if (moveUp > 0) {
        seq += `\x1b[${moveUp}A`;
    }

    seq += "\r";
    if (cursorCol > 0) {
        seq += `\x1b[${cursorCol}C`;
    }

    return {
        sequence: seq,
        totalRows,
        cursorRow,
        cursorCol,
    };
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
    const builtins = Object.keys(docsData).sort((a, b) => b.length - a.length).map(k => k.replace(/\./g, "\\."));
    const builtinsPattern = builtins.join("|");

    // Order matters for regex capture groups
    const regex = new RegExp([
        // 1. f-Strings (f"..." or f'...')
        /(f(?:(["'])(?:\\.|[^\\])*?\2))/.source,
        // 3. Strings (single/double quoted)
        /((["'])(?:\\.|[^\\])*?\4)/.source,
        // 5. Comments (#...)
        /(#.*)/.source,
        // 6. Keywords
        /(\b(?:def|class|import|from|return|if|else|elif|while|for|try|except|finally|with|as|pass|break|continue|lambda|yield|global|nonlocal|assert|del|raise)\b)/.source,
        // 7. Operands
        /(\b(?:is|not|in|and|or|True|False|None)\b)/.source,
        // 8. Built-ins / Methods
        `(\\b(?:${builtinsPattern})\\b)`,
        // 9. Numbers
        /(\b\d+\b)/.source,
        // 10. Punctuation
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
            // f-String
            const replaced = text.replace(/\{\{|\}\}|\{[^}]*\}/g, (m) => {
                if (m === '{{' || m === '}}') return m;
                const inner = m.slice(1, -1);
                return `${RESET}${COLOR_PUNCTUATION_1}{${RESET}${highlightPythonSyntax(inner)}${COLOR_PUNCTUATION_1}}${RESET}${COLOR_STRING}`;
            });
            result += `${COLOR_STRING}${replaced}${RESET}`;
        } else if (match[3]) {
            // String
            result += `${COLOR_STRING}${text}${RESET}`;
        } else if (match[5]) {
            // Comment
            result += `${COLOR_COMMENT}${text}${RESET}`;
        } else if (match[6]) {
            // Keyword
            result += `${COLOR_KEYWORD}${text}${RESET}`;
        } else if (match[7]) {
            // Operand
            result += `${COLOR_OPERAND}${text}${RESET}`;
        } else if (match[8]) {
            // Built-in / Method
            result += `${COLOR_METHOD}${text}${RESET}`;
        } else if (match[9]) {
            // Number
            result += `${COLOR_NUMBER}${text}${RESET}`;
        } else if (match[10]) {
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
