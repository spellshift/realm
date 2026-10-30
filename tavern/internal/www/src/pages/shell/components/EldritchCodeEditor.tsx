import React, { useState, useRef, useEffect, useMemo, useCallback } from "react";
import { AlertCircle, AlertTriangle, CheckCircle2, Wand2 } from "lucide-react";
import {
    analyzeEldritchCode,
    completeEldritchCode,
    EldritchCompletionItem,
    EldritchDiagnostic,
    HighlightTokenType,
    tokenizeEldritchLine,
} from "../utils/eldritchLsp";

export const MIN_EDITOR_HEIGHT = 96;
export const MAX_EDITOR_HEIGHT = 560;
export const DEFAULT_EDITOR_HEIGHT = 176;

export const TOKEN_COLOR_CLASSES: Record<HighlightTokenType, string> = {
    plain: "text-gray-100",
    keyword: "text-purple-400 font-semibold",
    boolean: "text-sky-400 font-semibold",
    module: "text-teal-300",
    function: "text-amber-200",
    builtin: "text-cyan-300",
    string: "text-orange-300",
    number: "text-emerald-300",
    comment: "text-gray-500 italic",
    operator: "text-pink-300",
    punctuation: "text-yellow-200/90",
};

interface HistoryEntry {
    code: string;
    selectionStart: number;
    selectionEnd: number;
}

export function useResizableEditorHeight(initialHeight: number = DEFAULT_EDITOR_HEIGHT) {
    const [editorHeight, setEditorHeight] = useState<number>(initialHeight);
    const isResizingRef = useRef<boolean>(false);
    const resizeStartYRef = useRef<number>(0);
    const resizeStartHeightRef = useRef<number>(initialHeight);

    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (!isResizingRef.current) return;
            const deltaY = resizeStartYRef.current - e.clientY;
            const nextHeight = Math.max(
                MIN_EDITOR_HEIGHT,
                Math.min(MAX_EDITOR_HEIGHT, resizeStartHeightRef.current + deltaY)
            );
            setEditorHeight(nextHeight);
        };

        const handleMouseUp = () => {
            if (isResizingRef.current) {
                isResizingRef.current = false;
                document.body.style.cursor = "";
                document.body.style.userSelect = "";
            }
        };

        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("mouseup", handleMouseUp);
        return () => {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("mouseup", handleMouseUp);
        };
    }, []);

    const handleResizeMouseDown = useCallback(
        (e: React.MouseEvent) => {
            e.preventDefault();
            isResizingRef.current = true;
            resizeStartYRef.current = e.clientY;
            resizeStartHeightRef.current = editorHeight;
            document.body.style.cursor = "ns-resize";
            document.body.style.userSelect = "none";
        },
        [editorHeight]
    );

    const handleResizeKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === "ArrowUp") {
            e.preventDefault();
            setEditorHeight((h) => Math.min(MAX_EDITOR_HEIGHT, h + 24));
        } else if (e.key === "ArrowDown") {
            e.preventDefault();
            setEditorHeight((h) => Math.max(MIN_EDITOR_HEIGHT, h - 24));
        }
    }, []);

    return {
        editorHeight,
        handleResizeMouseDown,
        handleResizeKeyDown,
    };
}

export interface EldritchLspStatusButtonProps {
    diagnostics: EldritchDiagnostic[];
    onClick: () => void;
    testId: string;
    includeWarningSubcountOnError?: boolean;
}

export const EldritchLspStatusButton: React.FC<EldritchLspStatusButtonProps> = ({
    diagnostics,
    onClick,
    testId,
    includeWarningSubcountOnError = false,
}) => {
    const [showPopover, setShowPopover] = useState<boolean>(false);
    const isHoveredRef = useRef<boolean>(false);

    const errorCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "error").length,
        [diagnostics]
    );
    const warningCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "warning").length,
        [diagnostics]
    );

    useEffect(() => {
        if (errorCount > 0 || warningCount > 0) {
            isHoveredRef.current = false;
            setShowPopover(false);
        }
    }, [errorCount, warningCount]);

    if (errorCount > 0) {
        return (
            <button
                type="button"
                data-testid={testId}
                onClick={onClick}
                className="inline-flex items-center gap-1 rounded-md text-xs font-mono transition-colors px-2 py-0.5 bg-red-500/15 text-red-300 hover:bg-red-500/25"
                title="eldritch-lsp live diagnostics"
            >
                <AlertCircle className="w-3.5 h-3.5 text-red-400" />
                <span>
                    {errorCount} {errorCount === 1 ? "error" : "errors"}
                    {includeWarningSubcountOnError && warningCount > 0
                        ? `, ${warningCount} warn`
                        : ""}
                </span>
            </button>
        );
    }

    if (warningCount > 0) {
        return (
            <button
                type="button"
                data-testid={testId}
                onClick={onClick}
                className="inline-flex items-center gap-1 rounded-md text-xs font-mono transition-colors px-2 py-0.5 bg-amber-500/15 text-amber-200 hover:bg-amber-500/25"
                title="eldritch-lsp live diagnostics"
            >
                <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                <span>
                    {warningCount} {warningCount === 1 ? "warning" : "warnings"}
                </span>
            </button>
        );
    }

    return (
        <div
            className="relative inline-flex items-center"
            onMouseEnter={() => {
                isHoveredRef.current = true;
                setShowPopover(true);
            }}
            onMouseLeave={() => {
                isHoveredRef.current = false;
                setShowPopover(false);
            }}
        >
            <button
                type="button"
                data-testid={testId}
                aria-label="No errors"
                onMouseEnter={() => {
                    isHoveredRef.current = true;
                    setShowPopover(true);
                }}
                onMouseLeave={() => {
                    isHoveredRef.current = false;
                    setShowPopover(false);
                }}
                onClick={() => {
                    if (!isHoveredRef.current) {
                        setShowPopover((prev) => !prev);
                    }
                    onClick();
                }}
                className="inline-flex items-center gap-1 rounded-md text-xs font-mono transition-colors p-1 text-emerald-400 hover:bg-gray-800"
            >
                <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            </button>
            {showPopover && (
                <div
                    data-testid={`${testId}-popover`}
                    role="tooltip"
                    className="absolute left-0 top-full mt-1 z-30 whitespace-nowrap rounded-md bg-gray-900 border border-gray-700 px-2 py-1 text-xs text-gray-200 shadow-lg pointer-events-none"
                >
                    No errors
                </div>
            )}
        </div>
    );
};

interface HighlightedLineContentProps {
    isEmptyFirstLine: boolean;
    beforeTokens: { type: HighlightTokenType; text: string }[];
    afterTokens: { type: HighlightTokenType; text: string }[];
    ghostSuffix: string | null;
    inlineCompletionTestId?: string;
}

const HighlightedLineContent: React.FC<HighlightedLineContentProps> = ({
    isEmptyFirstLine,
    beforeTokens,
    afterTokens,
    ghostSuffix,
    inlineCompletionTestId,
}) => {
    if (isEmptyFirstLine) {
        return <span className="text-gray-500 italic"># Write or paste Eldritch code...</span>;
    }

    if (beforeTokens.length === 0 && !ghostSuffix) {
        return <>{"\u200B"}</>;
    }

    return (
        <>
            {beforeTokens.map((tok, tokIdx) => (
                <span key={`b-${tokIdx}`} className={TOKEN_COLOR_CLASSES[tok.type]}>
                    {tok.text}
                </span>
            ))}
            {ghostSuffix && (
                <span
                    data-testid={inlineCompletionTestId}
                    className="text-gray-500/80 italic"
                >
                    {ghostSuffix}
                </span>
            )}
            {afterTokens.map((tok, tokIdx) => (
                <span key={`a-${tokIdx}`} className={TOKEN_COLOR_CLASSES[tok.type]}>
                    {tok.text}
                </span>
            ))}
        </>
    );
};

export interface EldritchCodeEditorProps {
    code: string;
    onChange: (nextCode: string) => void;
    onRunCode?: (code: string) => void;
    disabled?: boolean;
    editorHeight: number;
    fontSize?: number;
    ariaLabel: string;
    testIdPrefix: "composer" | "session-settings";
    showDiagnosticsPanel: boolean;
    autoFocus?: boolean;
    textareaRefExternal?: React.RefObject<HTMLTextAreaElement>;
}

const EldritchCodeEditor: React.FC<EldritchCodeEditorProps> = ({
    code,
    onChange,
    onRunCode,
    disabled = false,
    editorHeight,
    fontSize,
    ariaLabel,
    testIdPrefix,
    showDiagnosticsPanel,
    autoFocus = false,
    textareaRefExternal,
}) => {
    const internalTextareaRef = useRef<HTMLTextAreaElement>(null);
    const textareaRef = textareaRefExternal || internalTextareaRef;
    const highlightPreRef = useRef<HTMLPreElement>(null);
    const gutterRef = useRef<HTMLDivElement>(null);
    const lineHeightPx = fontSize !== undefined ? Math.max(10, Math.round(fontSize * 1.5)) : 20;

    // Cross-platform Undo / Redo history stack
    const historyRef = useRef<HistoryEntry[]>([
        { code, selectionStart: code.length, selectionEnd: code.length },
    ]);
    const historyIndexRef = useRef<number>(0);
    const pendingSelectionRef = useRef<{ start: number; end: number } | null>(null);
    const [, setHistoryVersion] = useState<number>(0);

    // Autocomplete & cursor state
    const [cursorOffset, setCursorOffset] = useState<number>(code.length);
    const [selectedCompletionIdx, setSelectedCompletionIdx] = useState<number>(0);
    const [dismissedCompletionAt, setDismissedCompletionAt] = useState<number | null>(null);
    const [hoveredDiagLine, setHoveredDiagLine] = useState<number | null>(null);
    const [editorScrollTop, setEditorScrollTop] = useState<number>(0);

    // Keep undo stack synced when parent updates `code` externally
    useEffect(() => {
        const currentEntry = historyRef.current[historyIndexRef.current];
        if (!currentEntry || currentEntry.code !== code) {
            const nextEntries = historyRef.current.slice(0, historyIndexRef.current + 1);
            nextEntries.push({
                code,
                selectionStart: code.length,
                selectionEnd: code.length,
            });
            if (nextEntries.length > 150) {
                nextEntries.shift();
            }
            historyRef.current = nextEntries;
            historyIndexRef.current = nextEntries.length - 1;
            setHistoryVersion((v) => v + 1);
        }
    }, [code]);

    // Apply pending cursor selection after programmatic edits / undo / redo
    useEffect(() => {
        if (pendingSelectionRef.current && textareaRef.current) {
            const { start, end } = pendingSelectionRef.current;
            pendingSelectionRef.current = null;
            textareaRef.current.selectionStart = start;
            textareaRef.current.selectionEnd = end;
            setCursorOffset(end);
        }
    }, [code, textareaRef]);

    useEffect(() => {
        if (autoFocus) {
            setTimeout(() => textareaRef.current?.focus(), 20);
        }
    }, [autoFocus, textareaRef]);

    const commitCodeChange = useCallback(
        (nextCode: string, selectionStart: number, selectionEnd: number = selectionStart) => {
            const nextEntries = historyRef.current.slice(0, historyIndexRef.current + 1);
            nextEntries.push({
                code: nextCode,
                selectionStart,
                selectionEnd,
            });
            if (nextEntries.length > 150) {
                nextEntries.shift();
            }
            historyRef.current = nextEntries;
            historyIndexRef.current = nextEntries.length - 1;
            pendingSelectionRef.current = { start: selectionStart, end: selectionEnd };
            setDismissedCompletionAt(null);
            setHistoryVersion((v) => v + 1);
            onChange(nextCode);
        },
        [onChange]
    );

    const handleUndo = useCallback(() => {
        if (historyIndexRef.current <= 0) return;
        historyIndexRef.current -= 1;
        const entry = historyRef.current[historyIndexRef.current];
        pendingSelectionRef.current = {
            start: entry.selectionStart,
            end: entry.selectionEnd,
        };
        setHistoryVersion((v) => v + 1);
        onChange(entry.code);
    }, [onChange]);

    const handleRedo = useCallback(() => {
        if (historyIndexRef.current >= historyRef.current.length - 1) return;
        historyIndexRef.current += 1;
        const entry = historyRef.current[historyIndexRef.current];
        pendingSelectionRef.current = {
            start: entry.selectionStart,
            end: entry.selectionEnd,
        };
        setHistoryVersion((v) => v + 1);
        onChange(entry.code);
    }, [onChange]);

    // Live eldritch-lsp diagnostics & inline completions
    const diagnostics = useMemo(() => analyzeEldritchCode(code), [code]);

    const diagnosticsByLine = useMemo(() => {
        const map = new Map<number, EldritchDiagnostic[]>();
        for (const d of diagnostics) {
            const arr = map.get(d.line) || [];
            arr.push(d);
            map.set(d.line, arr);
        }
        return map;
    }, [diagnostics]);

    const completionResult = useMemo(() => {
        if (dismissedCompletionAt === cursorOffset) return null;
        return completeEldritchCode(code, cursorOffset);
    }, [code, cursorOffset, dismissedCompletionAt]);

    const activeCompletions = completionResult?.items || [];

    useEffect(() => {
        setSelectedCompletionIdx(0);
    }, [completionResult?.replacePrefix, completionResult?.startOffset]);

    const selectedCompletionItem: EldritchCompletionItem | null =
        activeCompletions.length > 0
            ? activeCompletions[selectedCompletionIdx] || activeCompletions[0]
            : null;

    // Compute inline ghost completion suffix and cursor line/column for VSCode-style inline ghost text
    const inlineGhostInfo = useMemo(() => {
        if (!selectedCompletionItem || !completionResult) return null;
        const prefix = completionResult.replacePrefix || "";
        const insertText = selectedCompletionItem.insertText;
        const suffix = insertText.toLowerCase().startsWith(prefix.toLowerCase())
            ? insertText.slice(prefix.length)
            : insertText;
        if (!suffix) return null;

        const beforeCursor = code.slice(0, cursorOffset);
        const beforeLines = beforeCursor.split("\n");
        const lineIdx = beforeLines.length - 1;
        const colIdx = beforeLines[lineIdx].length;

        return {
            suffix,
            lineIdx,
            colIdx,
            item: selectedCompletionItem,
            totalCount: activeCompletions.length,
            selectedIndex: selectedCompletionIdx,
        };
    }, [
        selectedCompletionItem,
        completionResult,
        code,
        cursorOffset,
        activeCompletions.length,
        selectedCompletionIdx,
    ]);

    const applyCompletionItem = useCallback(
        (item: EldritchCompletionItem) => {
            if (!completionResult) return;
            const before = code.slice(0, completionResult.startOffset);
            const after = code.slice(completionResult.endOffset);
            const nextCode = before + item.insertText + after;
            const nextCursor = completionResult.startOffset + item.insertText.length;
            commitCodeChange(nextCode, nextCursor, nextCursor);
            setDismissedCompletionAt(nextCursor);
            textareaRef.current?.focus();
        },
        [code, completionResult, commitCodeChange, textareaRef]
    );

    const applyQuickFix = useCallback(
        (diag: EldritchDiagnostic) => {
            if (!diag.replacement) return;
            const lines = code.split("\n");
            const targetLine = lines[diag.replacement.line];
            if (targetLine === undefined) return;
            lines[diag.replacement.line] =
                targetLine.slice(0, diag.replacement.startCol) +
                diag.replacement.newText +
                targetLine.slice(diag.replacement.endCol);
            const nextCode = lines.join("\n");
            commitCodeChange(nextCode, nextCode.length, nextCode.length);
        },
        [code, commitCodeChange]
    );

    const toggleCommentSelection = useCallback(
        (start: number, end: number) => {
            const lineStart = code.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
            const lineEndIdx = code.indexOf("\n", end);
            const lineEnd = lineEndIdx === -1 ? code.length : lineEndIdx;
            const block = code.slice(lineStart, lineEnd);
            const blockLines = block.split("\n");
            const allCommented = blockLines
                .filter((l) => l.trim().length > 0)
                .every((l) => /^\s*#/.test(l));

            const transformed = blockLines.map((line) => {
                if (!line.trim()) return line;
                if (allCommented) {
                    return line.replace(/^(\s*)#\s?/, "$1");
                }
                const indentMatch = line.match(/^(\s*)/);
                const indent = indentMatch ? indentMatch[1] : "";
                return `${indent}# ${line.slice(indent.length)}`;
            });

            const newBlock = transformed.join("\n");
            const nextCode = code.slice(0, lineStart) + newBlock + code.slice(lineEnd);
            const delta = newBlock.length - block.length;
            commitCodeChange(nextCode, start, Math.max(start, end + delta));
        },
        [code, commitCodeChange]
    );

    const lines = code.split("\n");
    const lineCount = Math.max(1, lines.length);

    const handleScroll = (e: React.UIEvent<HTMLTextAreaElement>) => {
        const top = e.currentTarget.scrollTop;
        const left = e.currentTarget.scrollLeft;
        setEditorScrollTop(top);
        if (highlightPreRef.current) {
            highlightPreRef.current.scrollTop = top;
            highlightPreRef.current.scrollLeft = left;
        }
        if (gutterRef.current) {
            gutterRef.current.scrollTop = top;
        }
    };

    const handleTextareaMouseMove = (e: React.MouseEvent<HTMLTextAreaElement>) => {
        if (diagnostics.length === 0) {
            if (hoveredDiagLine !== null) {
                setHoveredDiagLine(null);
            }
            return;
        }
        const rect = e.currentTarget.getBoundingClientRect();
        if (rect.height === 0 && e.clientY === 0) {
            setHoveredDiagLine(diagnostics[0].line);
            return;
        }
        const offsetY = e.clientY - rect.top + e.currentTarget.scrollTop - 8;
        const lineIdx = Math.max(0, Math.floor(offsetY / lineHeightPx));
        if (diagnosticsByLine.has(lineIdx)) {
            setHoveredDiagLine(lineIdx);
        } else {
            setHoveredDiagLine(null);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        const isMod = e.ctrlKey || e.metaKey;
        const target = e.currentTarget;
        const start = target.selectionStart;
        const end = target.selectionEnd;

        // 1. Ctrl+Enter / Cmd+Enter -> Run script (if onRunCode is provided)
        if (e.key === "Enter" && isMod && onRunCode) {
            e.preventDefault();
            if (code.trim() && !disabled) {
                onRunCode(code);
            }
            return;
        }

        // 2. Undo (Ctrl+Z / Cmd+Z) & Redo (Ctrl+Shift+Z / Cmd+Shift+Z / Ctrl+Y / Cmd+Y)
        if (isMod && !e.altKey && e.key.toLowerCase() === "z") {
            e.preventDefault();
            if (e.shiftKey) {
                handleRedo();
            } else {
                handleUndo();
            }
            return;
        }
        if (isMod && !e.altKey && !e.shiftKey && e.key.toLowerCase() === "y") {
            e.preventDefault();
            handleRedo();
            return;
        }

        // 3. Toggle comment (Ctrl+/ or Cmd+/)
        if (isMod && e.key === "/") {
            e.preventDefault();
            toggleCommentSelection(start, end);
            return;
        }

        // 4. Explicit autocomplete trigger (Ctrl+Space)
        if (e.ctrlKey && e.code === "Space") {
            e.preventDefault();
            setDismissedCompletionAt(null);
            setCursorOffset(end);
            return;
        }

        // 5. VSCode-style inline autocomplete keyboard handling when completions are available
        if (activeCompletions.length > 0 && start === end) {
            const visibleCount = Math.min(8, activeCompletions.length);
            if ((e.key === "Tab" && !e.shiftKey) || (e.key === "Enter" && !isMod && !e.shiftKey)) {
                e.preventDefault();
                applyCompletionItem(activeCompletions[selectedCompletionIdx] || activeCompletions[0]);
                return;
            }
            if (!isMod && !e.altKey && !e.shiftKey && (e.key === "ArrowRight" || e.key === "ArrowDown")) {
                e.preventDefault();
                setSelectedCompletionIdx((idx) => (idx + 1) % visibleCount);
                return;
            }
            if (
                (e.key === "Tab" && e.shiftKey) ||
                (!isMod && !e.altKey && !e.shiftKey && (e.key === "ArrowLeft" || e.key === "ArrowUp"))
            ) {
                e.preventDefault();
                setSelectedCompletionIdx((idx) => (idx - 1 + visibleCount) % visibleCount);
                return;
            }
            if (e.key === "Escape") {
                e.preventDefault();
                setDismissedCompletionAt(cursorOffset);
                return;
            }
        }

        // 6. Tab / Shift+Tab -> Indent / Unindent by 4 spaces
        if (e.key === "Tab") {
            e.preventDefault();

            if (start !== end || e.shiftKey) {
                const lineStart = code.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
                const lineEndIdx = code.indexOf("\n", end);
                const lineEnd = lineEndIdx === -1 ? code.length : lineEndIdx;
                const block = code.slice(lineStart, lineEnd);
                const blockLines = block.split("\n");

                if (e.shiftKey) {
                    let removedFirst = 0;
                    let totalRemoved = 0;
                    const updatedLines = blockLines.map((line, idx) => {
                        const match = line.match(/^( {1,4}|\t)/);
                        const removeLen = match ? match[0].length : 0;
                        if (idx === 0) removedFirst = removeLen;
                        totalRemoved += removeLen;
                        return line.slice(removeLen);
                    });
                    const nextCode =
                        code.slice(0, lineStart) + updatedLines.join("\n") + code.slice(lineEnd);
                    commitCodeChange(
                        nextCode,
                        Math.max(lineStart, start - removedFirst),
                        Math.max(lineStart, end - totalRemoved)
                    );
                } else {
                    const updatedLines = blockLines.map((line) => `    ${line}`);
                    const nextCode =
                        code.slice(0, lineStart) + updatedLines.join("\n") + code.slice(lineEnd);
                    commitCodeChange(nextCode, start + 4, end + 4 * blockLines.length);
                }
                return;
            }

            const nextCode = code.slice(0, start) + "    " + code.slice(end);
            commitCodeChange(nextCode, start + 4, start + 4);
            return;
        }

        // 7. Smart auto-indent on Enter
        if (e.key === "Enter" && !isMod && !e.shiftKey) {
            e.preventDefault();
            const lineStart = code.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
            const currentLineBeforeCursor = code.slice(lineStart, start);
            const indentMatch = currentLineBeforeCursor.match(/^(\s*)/);
            const baseIndent = indentMatch ? indentMatch[1] : "";
            const trimmedBefore = currentLineBeforeCursor.replace(/#.*$/, "").trimEnd();
            const extraIndent = trimmedBefore.endsWith(":") ? "    " : "";
            const insertion = `\n${baseIndent}${extraIndent}`;
            const nextCode = code.slice(0, start) + insertion + code.slice(end);
            const nextPos = start + insertion.length;
            commitCodeChange(nextCode, nextPos, nextPos);
            return;
        }

        // 8. Smart 4-space dedent on Backspace
        if (e.key === "Backspace" && start === end && start > 0) {
            const lineStart = code.lastIndexOf("\n", start - 1) + 1;
            const beforeOnLine = code.slice(lineStart, start);
            if (
                beforeOnLine.length >= 4 &&
                /^ +$/.test(beforeOnLine) &&
                beforeOnLine.length % 4 === 0
            ) {
                e.preventDefault();
                const nextCode = code.slice(0, start - 4) + code.slice(start);
                commitCodeChange(nextCode, start - 4, start - 4);
                return;
            }
        }
    };

    const hoveredLineDiags =
        hoveredDiagLine !== null ? diagnosticsByLine.get(hoveredDiagLine) || null : null;

    return (
        <div
            onMouseLeave={() => setHoveredDiagLine(null)}
            className="relative flex bg-[#0f1118] font-mono text-xs overflow-hidden"
            style={{ height: `${editorHeight}px` }}
        >
            {/* Line Numbers & Gutter Diagnostic Markers */}
            <div
                ref={gutterRef}
                aria-hidden="true"
                className="select-none py-2 px-2 bg-gray-900 border-r border-gray-800 text-right text-gray-500 leading-5 min-w-[2.5rem] overflow-hidden"
                style={
                    fontSize !== undefined
                        ? { fontSize: `${fontSize}px`, lineHeight: `${lineHeightPx}px` }
                        : undefined
                }
            >
                {Array.from({ length: lineCount }).map((_, idx) => {
                    const lineDiags = diagnosticsByLine.get(idx);
                    const hasErr = Boolean(lineDiags?.some((d) => d.severity === "error"));
                    const hasWarn = Boolean(lineDiags?.some((d) => d.severity === "warning"));
                    let numberColorClass = "";
                    if (hasErr) {
                        numberColorClass = "text-red-400 font-semibold";
                    } else if (hasWarn) {
                        numberColorClass = "text-amber-300";
                    }

                    return (
                        <div
                            key={idx}
                            onMouseEnter={() => {
                                if (lineDiags && lineDiags.length > 0) {
                                    setHoveredDiagLine(idx);
                                }
                            }}
                            className="flex items-center justify-end gap-1 h-5"
                            style={
                                fontSize !== undefined ? { height: `${lineHeightPx}px` } : undefined
                            }
                            title={lineDiags?.map((d) => d.message).join("\n")}
                        >
                            {hasErr && (
                                <span className="w-1.5 h-1.5 rounded-full bg-red-400 inline-block" />
                            )}
                            {!hasErr && hasWarn && (
                                <span className="w-1.5 h-1.5 rounded-full bg-amber-400 inline-block" />
                            )}
                            <span className={numberColorClass}>{idx + 1}</span>
                        </div>
                    );
                })}
            </div>

            {/* Synchronized Syntax Highlighting Overlay + Inline Ghost Completion + Textarea */}
            <div className="relative flex-1 h-full overflow-hidden">
                <pre
                    ref={highlightPreRef}
                    aria-hidden="true"
                    data-testid={`${testIdPrefix}-syntax-highlight`}
                    className="pointer-events-none absolute inset-0 m-0 py-2 px-3 font-mono text-xs leading-5 whitespace-pre overflow-hidden"
                    style={
                        fontSize !== undefined
                            ? { fontSize: `${fontSize}px`, lineHeight: `${lineHeightPx}px` }
                            : undefined
                    }
                >
                    {lines.map((line, lineIdx) => {
                        const lineDiags = diagnosticsByLine.get(lineIdx);
                        const hasErr = Boolean(lineDiags?.some((d) => d.severity === "error"));
                        const hasWarn = Boolean(lineDiags?.some((d) => d.severity === "warning"));
                        const showGhostOnLine =
                            inlineGhostInfo && inlineGhostInfo.lineIdx === lineIdx;

                        const beforeGhostText = showGhostOnLine
                            ? line.slice(0, inlineGhostInfo.colIdx)
                            : line;
                        const afterGhostText = showGhostOnLine
                            ? line.slice(inlineGhostInfo.colIdx)
                            : "";

                        const beforeTokens = tokenizeEldritchLine(beforeGhostText);
                        const afterTokens = afterGhostText
                            ? tokenizeEldritchLine(afterGhostText)
                            : [];

                        let lineDecorationClass = "";
                        if (hasErr) {
                            lineDecorationClass =
                                "bg-red-500/10 decoration-red-400 decoration-wavy underline";
                        } else if (hasWarn) {
                            lineDecorationClass =
                                "bg-amber-500/10 decoration-amber-400 decoration-wavy underline";
                        }

                        return (
                            <div
                                key={lineIdx}
                                data-testid={
                                    testIdPrefix === "composer" && (hasErr || hasWarn)
                                        ? "composer-errored-line"
                                        : undefined
                                }
                                className={`min-h-[1.25rem] ${lineDecorationClass}`}
                                style={
                                    fontSize !== undefined
                                        ? {
                                              minHeight: `${lineHeightPx}px`,
                                              lineHeight: `${lineHeightPx}px`,
                                          }
                                        : undefined
                                }
                            >
                                <HighlightedLineContent
                                    isEmptyFirstLine={code.length === 0 && lineIdx === 0}
                                    beforeTokens={beforeTokens}
                                    afterTokens={afterTokens}
                                    ghostSuffix={showGhostOnLine ? inlineGhostInfo.suffix : null}
                                    inlineCompletionTestId={
                                        testIdPrefix === "composer"
                                            ? "composer-inline-completion"
                                            : undefined
                                    }
                                />
                            </div>
                        );
                    })}
                </pre>

                <textarea
                    ref={textareaRef}
                    aria-label={ariaLabel}
                    value={code}
                    disabled={disabled}
                    onScroll={handleScroll}
                    onMouseMove={handleTextareaMouseMove}
                    onMouseEnter={handleTextareaMouseMove}
                    onChange={(e) => {
                        const nextVal = e.target.value;
                        const selStart = e.target.selectionStart;
                        const selEnd = e.target.selectionEnd;
                        setCursorOffset(selEnd);
                        commitCodeChange(nextVal, selStart, selEnd);
                    }}
                    onKeyUp={(e) => setCursorOffset(e.currentTarget.selectionEnd)}
                    onClick={(e) => setCursorOffset(e.currentTarget.selectionEnd)}
                    onKeyDown={handleKeyDown}
                    placeholder="# Write or paste Eldritch code..."
                    spellCheck={false}
                    autoCapitalize="off"
                    autoComplete="off"
                    autoCorrect="off"
                    wrap="off"
                    className="relative z-10 w-full h-full bg-transparent text-transparent placeholder:text-transparent caret-slate-200 selection:bg-slate-400/25 py-2 px-3 leading-5 resize-none focus:outline-none font-mono text-xs overflow-auto"
                    style={
                        fontSize !== undefined
                            ? { fontSize: `${fontSize}px`, lineHeight: `${lineHeightPx}px` }
                            : undefined
                    }
                />

                {/* Hover Lint Popover on Errored Code */}
                {hoveredDiagLine !== null && hoveredLineDiags && hoveredLineDiags.length > 0 && (
                    <div
                        data-testid={`${testIdPrefix}-lint-popover`}
                        role="tooltip"
                        style={{
                            top: `${Math.min(
                                Math.max(4, (hoveredDiagLine + 1) * lineHeightPx + 6 - editorScrollTop),
                                Math.max(4, editorHeight - 56)
                            )}px`,
                            left: "12px",
                        }}
                        className="absolute z-30 max-w-lg rounded-md bg-gray-900 border border-gray-700 px-3 py-2 text-xs shadow-xl space-y-1.5"
                    >
                        {hoveredLineDiags.map((diag, idx) => (
                            <div
                                key={`${diag.line}-${diag.startCol}-${diag.code}-${idx}`}
                                className="flex items-start justify-between gap-3"
                            >
                                <div className="flex items-start gap-2 min-w-0">
                                    {diag.severity === "error" ? (
                                        <AlertCircle className="w-3.5 h-3.5 text-red-400 shrink-0 mt-0.5" />
                                    ) : (
                                        <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                                    )}
                                    <div className="space-y-0.5 min-w-0">
                                        <p className="text-gray-200 break-words leading-snug">
                                            {diag.message}
                                        </p>
                                        <span className="block font-mono text-[10px] text-gray-500">
                                            eldritch-lsp({diag.code}) • Ln {diag.line + 1}, Col{" "}
                                            {diag.startCol + 1}
                                        </span>
                                    </div>
                                </div>

                                {diag.replacement && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            applyQuickFix(diag);
                                            setHoveredDiagLine(null);
                                        }}
                                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-200 text-xs font-mono shrink-0 transition-colors"
                                    >
                                        <Wand2 className="w-3.5 h-3.5 text-gray-400" />
                                        {diag.replacement.label}
                                    </button>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* Bottom Diagnostics Bar */}
            {showDiagnosticsPanel && diagnostics.length > 0 && (
                <div
                    data-testid={`${testIdPrefix}-diagnostics-list`}
                    className="absolute bottom-0 left-0 right-0 z-20 max-h-28 overflow-y-auto divide-y divide-gray-800 bg-gray-900/95 backdrop-blur-sm border-t border-gray-800 text-xs shadow-lg"
                >
                    {diagnostics.map((diag, idx) => (
                        <div
                            key={`${diag.line}-${diag.startCol}-${diag.code}-${idx}`}
                            className="flex items-center justify-between gap-2 px-3 py-1"
                        >
                            <div className="flex items-center gap-2 min-w-0">
                                {diag.severity === "error" ? (
                                    <AlertCircle className="w-3.5 h-3.5 text-red-400 shrink-0" />
                                ) : (
                                    <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                                )}
                                <span className="font-mono text-xs text-gray-400 shrink-0">
                                    L{diag.line + 1}:
                                </span>
                                <span
                                    className={`truncate ${
                                        diag.severity === "error"
                                            ? "text-red-200"
                                            : "text-amber-200"
                                    }`}
                                >
                                    {diag.message}
                                </span>
                            </div>

                            {diag.replacement && (
                                <button
                                    type="button"
                                    onClick={() => applyQuickFix(diag)}
                                    className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-200 text-xs font-mono shrink-0 transition-colors"
                                >
                                    <Wand2 className="w-3.5 h-3.5 text-gray-400" />
                                    {diag.replacement.label}
                                </button>
                            )}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default EldritchCodeEditor;

