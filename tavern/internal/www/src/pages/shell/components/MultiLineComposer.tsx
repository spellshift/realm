import React, { useState, useRef, useEffect, useMemo, useCallback } from "react";
import {
    AlertCircle,
    AlertTriangle,
    CheckCircle2,
    Code2,
    GripHorizontal,
    Play,
    Redo2,
    Sparkles,
    Trash2,
    Undo2,
    Wand2,
    X,
} from "lucide-react";
import {
    analyzeEldritchCode,
    completeEldritchCode,
    EldritchCompletionItem,
    EldritchDiagnostic,
    getEldritchSignatureHelp,
    HighlightTokenType,
    tokenizeEldritchLine,
} from "../utils/eldritchLsp";

export interface ComposerSnippet {
    label: string;
    description: string;
    code: string;
}

export const ELDRITCH_SNIPPETS: ComposerSnippet[] = [
    {
        label: "Define Shell Aliases (ls, whoami, ps, pwd, cat, netstat)",
        description: "Register common shell helper functions in this Eldritch interpreter session",
        code: [
            'def ls(path="."):',
            "    return file.list(path)",
            "def whoami():",
            "    return user.info()",
            "def ps():",
            "    return process.list()",
            "def pwd():",
            "    return process.cwd()",
            "def cat(path):",
            "    return str(file.read(path))",
            "def netstat():",
            "    return net.conns()",
        ].join("\n"),
    },
    {
        label: "Open Interactive Portal",
        description: "Upgrade from queued callback mode to low-latency interactive mode",
        code: "portal.create()",
    },
    {
        label: "Host Situational Awareness",
        description: "Inspect OS, identity, network interfaces, and environment",
        code: `pprint(sys.info())\npprint(user.info())\ntprint(net.interfaces())`,
    },
    {
        label: "Filter Running Processes",
        description: "Loop over processes and print matching entries",
        code: `for p in process.list():\n    name = p["name"]\n    if "ssh" in name or "bash" in name:\n        print(f"{p['pid']}  {p['username']}  {name}")`,
    },
    {
        label: "Report Discovered Credential",
        description: "Save a discovered password directly into Tavern's Knowledge Graph",
        code: `report.user_password(username="admin", password="changeme")`,
    },
    {
        label: "Report SSH Private Key",
        description: "Read and report an SSH key into Tavern's Knowledge Graph",
        code: `key_data = str(file.read("/root/.ssh/id_rsa"))\nreport.ssh_key(username="root", key=key_data)`,
    },
];

const TOKEN_COLOR_CLASSES: Record<HighlightTokenType, string> = {
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

interface MultiLineComposerProps {
    isOpen: boolean;
    onToggle: () => void;
    code: string;
    onChange: (nextCode: string) => void;
    onRunCode: (code: string) => void;
    onStageToPrompt?: (code: string) => void;
    disabled?: boolean;
}

const MIN_EDITOR_HEIGHT = 96;
const MAX_EDITOR_HEIGHT = 560;
const DEFAULT_EDITOR_HEIGHT = 176;

const MultiLineComposer: React.FC<MultiLineComposerProps> = ({
    isOpen,
    onToggle,
    code,
    onChange,
    onRunCode,
    disabled = false,
}) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const highlightPreRef = useRef<HTMLPreElement>(null);
    const gutterRef = useRef<HTMLDivElement>(null);

    // Resizable height state
    const [editorHeight, setEditorHeight] = useState<number>(DEFAULT_EDITOR_HEIGHT);
    const isResizingRef = useRef<boolean>(false);
    const resizeStartYRef = useRef<number>(0);
    const resizeStartHeightRef = useRef<number>(DEFAULT_EDITOR_HEIGHT);

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
    const [showDiagnosticsPanel, setShowDiagnosticsPanel] = useState<boolean>(false);

    // Keep undo stack synced when parent updates `code` externally (e.g. "Load in Composer")
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
    }, [code]);

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => textareaRef.current?.focus(), 20);
        }
    }, [isOpen]);

    // Vertical resize mouse handlers
    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (!isResizingRef.current) return;
            // Dragging the top handle upwards increases composer height
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

    const handleResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        isResizingRef.current = true;
        resizeStartYRef.current = e.clientY;
        resizeStartHeightRef.current = editorHeight;
        document.body.style.cursor = "ns-resize";
        document.body.style.userSelect = "none";
    };

    const handleResizeKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "ArrowUp") {
            e.preventDefault();
            setEditorHeight((h) => Math.min(MAX_EDITOR_HEIGHT, h + 24));
        } else if (e.key === "ArrowDown") {
            e.preventDefault();
            setEditorHeight((h) => Math.max(MIN_EDITOR_HEIGHT, h - 24));
        }
    };

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

    // Live eldritch-lsp diagnostics, completions, and signature help
    const diagnostics = useMemo(() => analyzeEldritchCode(code), [code]);
    const errorCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "error").length,
        [diagnostics]
    );
    const warningCount = useMemo(
        () => diagnostics.filter((d) => d.severity === "warning").length,
        [diagnostics]
    );

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

    const signatureHelp = useMemo(
        () => getEldritchSignatureHelp(code, cursorOffset),
        [code, cursorOffset]
    );

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
        [code, completionResult, commitCodeChange]
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
            commitCodeChange(
                nextCode,
                start,
                Math.max(start, end + delta)
            );
        },
        [code, commitCodeChange]
    );

    if (!isOpen) {
        return null;
    }

    const lines = code.split("\n");
    const lineCount = Math.max(1, lines.length);

    const handleScroll = (e: React.UIEvent<HTMLTextAreaElement>) => {
        const top = e.currentTarget.scrollTop;
        const left = e.currentTarget.scrollLeft;
        if (highlightPreRef.current) {
            highlightPreRef.current.scrollTop = top;
            highlightPreRef.current.scrollLeft = left;
        }
        if (gutterRef.current) {
            gutterRef.current.scrollTop = top;
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        const isMod = e.ctrlKey || e.metaKey;
        const target = e.currentTarget;
        const start = target.selectionStart;
        const end = target.selectionEnd;

        // 1. Ctrl+Enter / Cmd+Enter -> Run script
        if (e.key === "Enter" && isMod) {
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

        // 5. Autocomplete keyboard navigation when completions are available:
        // Tab / Shift+Tab and Left / Right (or Up / Down) arrow keys cycle options; Enter accepts
        if (activeCompletions.length > 0 && start === end) {
            const visibleCount = Math.min(8, activeCompletions.length);
            if (
                (e.key === "Tab" && !e.shiftKey) ||
                (!isMod && !e.altKey && !e.shiftKey && (e.key === "ArrowRight" || e.key === "ArrowDown"))
            ) {
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
            if (e.key === "Enter" && !isMod && !e.shiftKey) {
                e.preventDefault();
                applyCompletionItem(activeCompletions[selectedCompletionIdx] || activeCompletions[0]);
                return;
            }
            if (e.key === "Escape") {
                e.preventDefault();
                setDismissedCompletionAt(cursorOffset);
                return;
            }
        }

        // 6. Tab / Shift+Tab -> Indent / Unindent by 4 spaces (single-line or multi-line selection)
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
                    commitCodeChange(
                        nextCode,
                        start + 4,
                        end + 4 * blockLines.length
                    );
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
            if (beforeOnLine.length >= 4 && /^ +$/.test(beforeOnLine) && beforeOnLine.length % 4 === 0) {
                e.preventDefault();
                const nextCode = code.slice(0, start - 4) + code.slice(start);
                commitCodeChange(nextCode, start - 4, start - 4);
                return;
            }
        }
    };

    const handleSnippetSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const idx = parseInt(e.target.value, 10);
        if (isNaN(idx) || !ELDRITCH_SNIPPETS[idx]) return;
        const snippet = ELDRITCH_SNIPPETS[idx];
        const existingTrimmed = code.replace(/\s+$/, "");
        const nextCode = existingTrimmed
            ? `${existingTrimmed}\n\n${snippet.code}`
            : snippet.code;
        commitCodeChange(nextCode, nextCode.length, nextCode.length);
        e.target.value = "";
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    const canUndo = historyIndexRef.current > 0;
    const canRedo = historyIndexRef.current < historyRef.current.length - 1;

    return (
        <div
            data-testid="multiline-composer"
            data-tour="shell-composer"
            className="flex flex-col rounded-md border border-[#262b38] bg-[#141822] shadow-lg overflow-hidden relative"
        >
            {/* Top Vertical Resize Drag Handle */}
            <div
                data-testid="composer-resize-handle"
                role="separator"
                aria-orientation="horizontal"
                aria-label="Resize composer"
                tabIndex={0}
                onMouseDown={handleResizeMouseDown}
                onKeyDown={handleResizeKeyDown}
                title="Drag to resize composer height"
                className="h-2 w-full bg-[#181c28] hover:bg-purple-500/30 border-b border-[#262b38] cursor-ns-resize flex items-center justify-center transition-colors group"
            >
                <GripHorizontal className="w-3.5 h-3.5 text-gray-600 group-hover:text-purple-300" />
            </div>

            {/* Composer Toolbar Header */}
            <div className="flex flex-wrap items-center justify-between gap-2 px-3 py-1.5 bg-[#181c28] border-b border-[#262b38]">
                <div className="flex items-center gap-2">
                    <span className="inline-flex items-center gap-1.5 text-xs font-semibold text-gray-200">
                        <Code2 className="w-3.5 h-3.5 text-purple-400" />
                        Composer
                    </span>

                    {/* Undo / Redo Controls */}
                    <div className="inline-flex items-center rounded border border-[#2f364a] bg-[#11131a]">
                        <button
                            type="button"
                            onClick={handleUndo}
                            disabled={!canUndo || disabled}
                            aria-label="Undo"
                            title="Undo (Ctrl+Z / Cmd+Z)"
                            className="p-1 text-gray-400 hover:text-gray-200 disabled:opacity-35 disabled:cursor-not-allowed transition-colors"
                        >
                            <Undo2 className="w-3.5 h-3.5" />
                        </button>
                        <button
                            type="button"
                            onClick={handleRedo}
                            disabled={!canRedo || disabled}
                            aria-label="Redo"
                            title="Redo (Ctrl+Shift+Z / Cmd+Shift+Z / Ctrl+Y)"
                            className="p-1 text-gray-400 hover:text-gray-200 border-l border-[#2f364a] disabled:opacity-35 disabled:cursor-not-allowed transition-colors"
                        >
                            <Redo2 className="w-3.5 h-3.5" />
                        </button>
                    </div>

                    {/* Live eldritch-lsp Status Pill */}
                    <button
                        type="button"
                        data-testid="eldritch-lsp-status"
                        onClick={() => setShowDiagnosticsPanel((prev) => !prev)}
                        className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-mono border transition-colors ${
                            errorCount > 0
                                ? "bg-red-500/15 border-red-500/40 text-red-300 hover:bg-red-500/25"
                                : warningCount > 0
                                  ? "bg-amber-500/15 border-amber-500/40 text-amber-200 hover:bg-amber-500/25"
                                  : "bg-emerald-500/10 border-emerald-500/25 text-emerald-300 hover:bg-emerald-500/20"
                        }`}
                        title="eldritch-lsp live diagnostics (click to toggle details)"
                    >
                        {errorCount > 0 ? (
                            <>
                                <AlertCircle className="w-3 h-3 text-red-400" />
                                <span>
                                    {errorCount} {errorCount === 1 ? "error" : "errors"}
                                    {warningCount > 0 ? `, ${warningCount} warn` : ""}
                                </span>
                            </>
                        ) : warningCount > 0 ? (
                            <>
                                <AlertTriangle className="w-3 h-3 text-amber-400" />
                                <span>
                                    {warningCount} {warningCount === 1 ? "warning" : "warnings"}
                                </span>
                            </>
                        ) : (
                            <>
                                <CheckCircle2 className="w-3 h-3 text-emerald-400" />
                                <span>eldritch-lsp</span>
                            </>
                        )}
                    </button>
                </div>

                <div className="flex items-center gap-2 flex-wrap">
                    {/* Snippets Selector */}
                    <div className="relative flex items-center">
                        <Sparkles className="w-3 h-3 text-purple-400 absolute left-2 pointer-events-none" />
                        <select
                            aria-label="Insert Eldritch snippet"
                            defaultValue=""
                            onChange={handleSnippetSelect}
                            className="pl-6 pr-6 py-1 rounded bg-[#11131a] border border-[#2f364a] hover:border-purple-500/50 text-xs text-gray-200 focus:outline-none focus:border-purple-500"
                        >
                            <option value="" disabled>
                                Insert snippet...
                            </option>
                            {ELDRITCH_SNIPPETS.map((snippet, idx) => (
                                <option key={snippet.label} value={idx}>
                                    {snippet.label}
                                </option>
                            ))}
                        </select>
                    </div>

                    {code.length > 0 && (
                        <button
                            type="button"
                            onClick={() => commitCodeChange("", 0, 0)}
                            className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs text-gray-400 hover:text-red-300 hover:bg-red-950/30 transition-colors"
                            title="Clear composer"
                        >
                            <Trash2 className="w-3 h-3" />
                            Clear
                        </button>
                    )}

                    <button
                        type="button"
                        disabled={!code.trim() || disabled}
                        onClick={() => onRunCode(code)}
                        className="inline-flex items-center gap-1.5 px-3 py-1 rounded bg-purple-600 hover:bg-purple-500 disabled:opacity-40 disabled:cursor-not-allowed text-xs font-semibold text-white transition-colors shadow-sm"
                        title="Execute script in shell (Ctrl+Enter / Cmd+Enter)"
                    >
                        <Play className="w-3 h-3 fill-current" />
                        Run
                    </button>

                    <button
                        type="button"
                        onClick={onToggle}
                        aria-label="Close composer"
                        title="Hide composer (Alt+M)"
                        className="p-1 rounded text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors"
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Editor Area with Gutter + Syntax Highlighting Layer + Transparent Textarea */}
            <div
                className="relative flex bg-[#0f1118] font-mono text-xs overflow-hidden"
                style={{ height: `${editorHeight}px` }}
            >
                {/* Line Numbers & Gutter Diagnostic Markers */}
                <div
                    ref={gutterRef}
                    aria-hidden="true"
                    className="select-none py-2 px-2 bg-[#12151e] border-r border-[#222736] text-right text-gray-600 leading-5 min-w-[2.5rem] overflow-hidden"
                >
                    {Array.from({ length: lineCount }).map((_, idx) => {
                        const lineDiags = diagnosticsByLine.get(idx);
                        const hasErr = lineDiags?.some((d) => d.severity === "error");
                        const hasWarn = lineDiags?.some((d) => d.severity === "warning");
                        return (
                            <div
                                key={idx}
                                className="flex items-center justify-end gap-1 h-5"
                                title={lineDiags?.map((d) => d.message).join("\n")}
                            >
                                {hasErr ? (
                                    <span className="w-1.5 h-1.5 rounded-full bg-red-400 inline-block" />
                                ) : hasWarn ? (
                                    <span className="w-1.5 h-1.5 rounded-full bg-amber-400 inline-block" />
                                ) : null}
                                <span className={hasErr ? "text-red-400 font-semibold" : hasWarn ? "text-amber-300" : ""}>
                                    {idx + 1}
                                </span>
                            </div>
                        );
                    })}
                </div>

                {/* Synchronized Syntax Highlighting Overlay + Textarea */}
                <div className="relative flex-1 h-full overflow-hidden">
                    <pre
                        ref={highlightPreRef}
                        aria-hidden="true"
                        data-testid="composer-syntax-highlight"
                        className="pointer-events-none absolute inset-0 m-0 py-2 px-3 font-mono text-xs leading-5 whitespace-pre overflow-hidden"
                    >
                        {lines.map((line, lineIdx) => {
                            const tokens = tokenizeEldritchLine(line);
                            const lineDiags = diagnosticsByLine.get(lineIdx);
                            const hasErr = lineDiags?.some((d) => d.severity === "error");
                            const hasWarn = lineDiags?.some((d) => d.severity === "warning");
                            return (
                                <div
                                    key={lineIdx}
                                    className={`min-h-[1.25rem] ${
                                        hasErr
                                            ? "bg-red-500/10 decoration-red-400 decoration-wavy underline"
                                            : hasWarn
                                              ? "bg-amber-500/10 decoration-amber-400 decoration-wavy underline"
                                              : ""
                                    }`}
                                >
                                    {tokens.length === 0 ? (
                                        "\u200B"
                                    ) : (
                                        tokens.map((tok, tokIdx) => (
                                            <span
                                                key={tokIdx}
                                                className={TOKEN_COLOR_CLASSES[tok.type]}
                                            >
                                                {tok.text}
                                            </span>
                                        ))
                                    )}
                                </div>
                            );
                        })}
                    </pre>

                    <textarea
                        ref={textareaRef}
                        aria-label="Multi-line Eldritch script editor"
                        value={code}
                        disabled={disabled}
                        onScroll={handleScroll}
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
                        placeholder="Write or paste multi-line Eldritch code... (Tab / ← → cycle completions, Enter accept, Ctrl/Cmd+Z undo, Ctrl/Cmd+Enter run)"
                        spellCheck={false}
                        autoCapitalize="off"
                        autoComplete="off"
                        autoCorrect="off"
                        wrap="off"
                        className="relative z-10 w-full h-full bg-transparent text-transparent caret-slate-200 selection:bg-slate-400/25 py-2 px-3 leading-5 resize-none focus:outline-none font-mono text-xs overflow-auto"
                    />
                </div>

                {/* Floating Diagnostics Overlay when user explicitly expands multiple diagnostics */}
                {showDiagnosticsPanel && diagnostics.length > 0 && (
                    <div className="absolute bottom-0 left-0 right-0 z-20 max-h-28 overflow-y-auto divide-y divide-[#222736] bg-[#131722]/95 backdrop-blur-sm border-t border-[#262b38] text-xs shadow-lg">
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
                                    <span className="font-mono text-[11px] text-gray-400 shrink-0">
                                        L{diag.line + 1}:
                                    </span>
                                    <span
                                        className={`truncate ${
                                            diag.severity === "error" ? "text-red-200" : "text-amber-200"
                                        }`}
                                    >
                                        {diag.message}
                                    </span>
                                </div>

                                {diag.replacement && (
                                    <button
                                        type="button"
                                        onClick={() => applyQuickFix(diag)}
                                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-200 text-[11px] font-mono shrink-0 transition-colors"
                                    >
                                        <Wand2 className="w-3 h-3" />
                                        {diag.replacement.label}
                                    </button>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* Constant Fixed-Height (h-8) Assist Bar — prevents any vertical layout shift when autocomplete or diagnostics appear/disappear */}
            <div className="h-8 flex items-center px-3 bg-[#161b28] border-t border-[#262b38] overflow-hidden text-xs shrink-0">
                {activeCompletions.length > 0 ? (
                    <div
                        data-testid="composer-autocomplete-list"
                        className="flex items-center gap-1.5 w-full overflow-x-auto no-scrollbar"
                    >
                        <span className="text-[10px] font-mono uppercase tracking-wider text-purple-300 shrink-0 mr-1">
                            Tab / ← → cycle • Enter accept:
                        </span>
                        {activeCompletions.slice(0, 8).map((item, idx) => {
                            const isSelected = idx === selectedCompletionIdx;
                            return (
                                <button
                                    key={item.label}
                                    type="button"
                                    onMouseDown={(e) => {
                                        e.preventDefault();
                                        applyCompletionItem(item);
                                    }}
                                    className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded font-mono text-xs shrink-0 transition-colors ${
                                        isSelected
                                            ? "bg-purple-600 text-white font-semibold"
                                            : "bg-[#1e2436] text-gray-300 hover:bg-[#272f46]"
                                    }`}
                                    title={item.documentation || item.detail}
                                >
                                    <span>{item.label}</span>
                                    <span
                                        className={`text-[10px] px-1 rounded ${
                                            isSelected
                                                ? "bg-purple-800/80 text-purple-100"
                                                : "bg-black/30 text-gray-400"
                                        }`}
                                    >
                                        {item.kind}
                                    </span>
                                </button>
                            );
                        })}
                    </div>
                ) : signatureHelp ? (
                    <div
                        data-testid="composer-signature-help"
                        className="flex items-center justify-between gap-2 w-full text-[11px] font-mono text-gray-300 truncate"
                    >
                        <div className="truncate">
                            <span className="text-teal-300 font-semibold">
                                {signatureHelp.functionName}
                            </span>
                            <span className="text-gray-500">(</span>
                            {signatureHelp.params.map((param, idx) => (
                                <React.Fragment key={param}>
                                    {idx > 0 && <span className="text-gray-500">, </span>}
                                    <span
                                        className={
                                            idx === signatureHelp.activeParam
                                                ? "text-amber-300 font-semibold underline underline-offset-2"
                                                : "text-gray-400"
                                        }
                                    >
                                        {param}
                                    </span>
                                </React.Fragment>
                            ))}
                            <span className="text-gray-500">)</span>
                        </div>
                        <span className="text-gray-400 font-sans truncate max-w-md hidden sm:inline">
                            {signatureHelp.description}
                        </span>
                    </div>
                ) : diagnostics.length > 0 ? (
                    <div
                        data-testid="composer-diagnostics-list"
                        className="flex items-center justify-between gap-2 w-full truncate"
                    >
                        <div className="flex items-center gap-2 min-w-0 truncate">
                            {diagnostics[0].severity === "error" ? (
                                <AlertCircle className="w-3.5 h-3.5 text-red-400 shrink-0" />
                            ) : (
                                <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                            )}
                            <span className="font-mono text-[11px] text-gray-400 shrink-0">
                                L{diagnostics[0].line + 1}:
                            </span>
                            <span
                                className={`truncate ${
                                    diagnostics[0].severity === "error" ? "text-red-200" : "text-amber-200"
                                }`}
                            >
                                {diagnostics[0].message}
                            </span>
                        </div>

                        {diagnostics[0].replacement && (
                            <button
                                type="button"
                                onClick={() => applyQuickFix(diagnostics[0])}
                                className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-200 text-[11px] font-mono shrink-0 transition-colors"
                            >
                                <Wand2 className="w-3 h-3" />
                                {diagnostics[0].replacement.label}
                            </button>
                        )}
                    </div>
                ) : (
                    <div className="flex items-center justify-between w-full text-[11px] text-gray-500 font-mono truncate">
                        <span>Tab / ← → cycle completions • Enter accept • Ctrl/Cmd+Z undo • Ctrl/Cmd+/ comment</span>
                        <span className="hidden sm:inline">Ctrl/Cmd+Enter to run</span>
                    </div>
                )}
            </div>
        </div>
    );
};

export default MultiLineComposer;
