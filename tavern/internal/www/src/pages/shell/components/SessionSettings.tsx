import React, { useState, useRef, useEffect, useMemo, useCallback } from "react";
import { BookOpenIcon } from "@heroicons/react/24/outline";
import {
    AlertCircle,
    AlertTriangle,
    ArrowRight,
    CheckCircle2,
    GripHorizontal,
    Search,
    Settings,
    Wand2,
    X,
} from "lucide-react";
import {
    analyzeEldritchCode,
    completeEldritchCode,
    EldritchCompletionItem,
    EldritchDiagnostic,
    HighlightTokenType,
    tokenizeEldritchLine,
} from "../utils/eldritchLsp";
import { ShellTomeItem } from "../hooks/useShellData";
import {
    DEFAULT_RC_ALIASES_SCRIPT,
    FALLBACK_ELDRITCH_TOMES,
    formatTomeForComposer,
    loadRcEnabled,
    loadRcScript,
    saveRcEnabled,
    saveRcScript,
} from "./MultiLineComposer";

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

export interface SessionSettingsProps {
    isOpen: boolean;
    onToggle?: () => void;
    onClose?: () => void;
    disabled?: boolean;
    tomes?: ShellTomeItem[];
    rcCode?: string;
    onRcCodeChange?: (nextRcCode: string) => void;
    onRcChange?: (nextRcCode: string) => void;
    rcEnabled?: boolean;
    onRcEnabledChange?: (enabled: boolean) => void;
}

const MIN_EDITOR_HEIGHT = 96;
const MAX_EDITOR_HEIGHT = 560;
const DEFAULT_EDITOR_HEIGHT = 176;

const SessionSettings: React.FC<SessionSettingsProps> = ({
    isOpen,
    onToggle,
    onClose,
    disabled = false,
    tomes,
    rcCode: controlledRcCode,
    onRcCodeChange,
    onRcChange,
    rcEnabled: controlledRcEnabled,
    onRcEnabledChange,
}) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const highlightPreRef = useRef<HTMLPreElement>(null);
    const gutterRef = useRef<HTMLDivElement>(null);
    const tomePickerRef = useRef<HTMLDivElement>(null);
    const loadTomeButtonRef = useRef<HTMLButtonElement>(null);
    const tomeSearchInputRef = useRef<HTMLInputElement>(null);

    const [internalRcCode, setInternalRcCode] = useState<string>(
        () => controlledRcCode ?? loadRcScript()
    );
    const [internalRcEnabled, setInternalRcEnabled] = useState<boolean>(
        () => controlledRcEnabled ?? loadRcEnabled()
    );
    const [isEditingScript, setIsEditingScript] = useState<boolean>(
        () => (controlledRcCode ?? loadRcScript()).trim().length > 0
    );

    useEffect(() => {
        if (controlledRcCode !== undefined) {
            setInternalRcCode(controlledRcCode);
        }
    }, [controlledRcCode]);

    useEffect(() => {
        if (controlledRcEnabled !== undefined) {
            setInternalRcEnabled(controlledRcEnabled);
        }
    }, [controlledRcEnabled]);

    const effectiveRcCode = internalRcCode;
    const effectiveRcEnabled = internalRcEnabled;

    const handleRcCodeChange = useCallback(
        (nextCode: string) => {
            setInternalRcCode(nextCode);
            saveRcScript(nextCode);
            onRcCodeChange?.(nextCode);
            onRcChange?.(nextCode);
        },
        [onRcCodeChange, onRcChange]
    );

    const handleSetRcEnabled = useCallback(
        (next: boolean) => {
            setInternalRcEnabled(next);
            saveRcEnabled(next);
            onRcEnabledChange?.(next);
        },
        [onRcEnabledChange]
    );

    const handleToggleRcEnabled = useCallback(() => {
        handleSetRcEnabled(!effectiveRcEnabled);
    }, [effectiveRcEnabled, handleSetRcEnabled]);

    // Searchable SHELL_RC Tome Picker state
    const [tomePickerOpen, setTomePickerOpen] = useState<boolean>(false);
    const [tomeSearchQuery, setTomeSearchQuery] = useState<string>("");

    const availableRcTomes = useMemo(() => {
        const source = tomes && tomes.length > 0 ? tomes : FALLBACK_ELDRITCH_TOMES;
        const shellRcOnly = source.filter((t) => t.tactic?.toUpperCase() === "SHELL_RC");
        if (shellRcOnly.length > 0) {
            return shellRcOnly;
        }
        return FALLBACK_ELDRITCH_TOMES.filter((t) => t.tactic?.toUpperCase() === "SHELL_RC");
    }, [tomes]);

    const shellAliasesTome = useMemo(
        () =>
            availableRcTomes.find((t) => t.name.trim().toLowerCase() === "shell aliases") ||
            availableRcTomes[0] ||
            null,
        [availableRcTomes]
    );

    const filteredRcTomes = useMemo(() => {
        const q = tomeSearchQuery.trim().toLowerCase();
        if (!q) return availableRcTomes;
        return availableRcTomes.filter(
            (t) =>
                t.name.toLowerCase().includes(q) ||
                (t.description && t.description.toLowerCase().includes(q)) ||
                t.eldritch.toLowerCase().includes(q)
        );
    }, [availableRcTomes, tomeSearchQuery]);

    useEffect(() => {
        if (!tomePickerOpen) return;
        const handleClickOutside = (e: MouseEvent) => {
            const target = e.target as Node;
            if (
                tomePickerRef.current &&
                !tomePickerRef.current.contains(target) &&
                (!loadTomeButtonRef.current || !loadTomeButtonRef.current.contains(target))
            ) {
                setTomePickerOpen(false);
            }
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, [tomePickerOpen]);

    useEffect(() => {
        if (tomePickerOpen) {
            setTimeout(() => tomeSearchInputRef.current?.focus({ preventScroll: true }), 10);
        } else {
            setTomeSearchQuery("");
        }
    }, [tomePickerOpen]);

    // Resizable height state
    const [editorHeight, setEditorHeight] = useState<number>(DEFAULT_EDITOR_HEIGHT);
    const isResizingRef = useRef<boolean>(false);
    const resizeStartYRef = useRef<number>(0);
    const resizeStartHeightRef = useRef<number>(DEFAULT_EDITOR_HEIGHT);

    // Cross-platform Undo / Redo history stack
    const historyRef = useRef<HistoryEntry[]>([
        {
            code: effectiveRcCode,
            selectionStart: effectiveRcCode.length,
            selectionEnd: effectiveRcCode.length,
        },
    ]);
    const historyIndexRef = useRef<number>(0);
    const pendingSelectionRef = useRef<{ start: number; end: number } | null>(null);
    const [, setHistoryVersion] = useState<number>(0);

    // Autocomplete & cursor state
    const [cursorOffset, setCursorOffset] = useState<number>(effectiveRcCode.length);
    const [selectedCompletionIdx, setSelectedCompletionIdx] = useState<number>(0);
    const [dismissedCompletionAt, setDismissedCompletionAt] = useState<number | null>(null);
    const [showDiagnosticsPanel, setShowDiagnosticsPanel] = useState<boolean>(true);
    const [hoveredDiagLine, setHoveredDiagLine] = useState<number | null>(null);
    const [editorScrollTop, setEditorScrollTop] = useState<number>(0);

    useEffect(() => {
        const currentEntry = historyRef.current[historyIndexRef.current];
        if (!currentEntry || currentEntry.code !== effectiveRcCode) {
            const nextEntries = historyRef.current.slice(0, historyIndexRef.current + 1);
            nextEntries.push({
                code: effectiveRcCode,
                selectionStart: effectiveRcCode.length,
                selectionEnd: effectiveRcCode.length,
            });
            if (nextEntries.length > 150) {
                nextEntries.shift();
            }
            historyRef.current = nextEntries;
            historyIndexRef.current = nextEntries.length - 1;
            setHistoryVersion((v) => v + 1);
        }
    }, [effectiveRcCode]);

    useEffect(() => {
        if (pendingSelectionRef.current && textareaRef.current) {
            const { start, end } = pendingSelectionRef.current;
            pendingSelectionRef.current = null;
            textareaRef.current.selectionStart = start;
            textareaRef.current.selectionEnd = end;
            setCursorOffset(end);
        }
    }, [effectiveRcCode]);

    useEffect(() => {
        if (isOpen && isEditingScript) {
            setTimeout(() => textareaRef.current?.focus(), 20);
        }
    }, [isOpen, isEditingScript]);

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
            handleRcCodeChange(nextCode);
        },
        [handleRcCodeChange]
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
        handleRcCodeChange(entry.code);
    }, [handleRcCodeChange]);

    const handleRedo = useCallback(() => {
        if (historyIndexRef.current >= historyRef.current.length - 1) return;
        historyIndexRef.current += 1;
        const entry = historyRef.current[historyIndexRef.current];
        pendingSelectionRef.current = {
            start: entry.selectionStart,
            end: entry.selectionEnd,
        };
        setHistoryVersion((v) => v + 1);
        handleRcCodeChange(entry.code);
    }, [handleRcCodeChange]);

    // Live eldritch-lsp diagnostics & inline completions
    const diagnostics = useMemo(() => analyzeEldritchCode(effectiveRcCode), [effectiveRcCode]);
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
        return completeEldritchCode(effectiveRcCode, cursorOffset);
    }, [effectiveRcCode, cursorOffset, dismissedCompletionAt]);

    const activeCompletions = completionResult?.items || [];

    useEffect(() => {
        setSelectedCompletionIdx(0);
    }, [completionResult?.replacePrefix, completionResult?.startOffset]);

    const selectedCompletionItem: EldritchCompletionItem | null =
        activeCompletions.length > 0
            ? activeCompletions[selectedCompletionIdx] || activeCompletions[0]
            : null;

    const inlineGhostInfo = useMemo(() => {
        if (!selectedCompletionItem || !completionResult) return null;
        const prefix = completionResult.replacePrefix || "";
        const insertText = selectedCompletionItem.insertText;
        const suffix = insertText.toLowerCase().startsWith(prefix.toLowerCase())
            ? insertText.slice(prefix.length)
            : insertText;
        if (!suffix) return null;

        const beforeCursor = effectiveRcCode.slice(0, cursorOffset);
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
        effectiveRcCode,
        cursorOffset,
        activeCompletions.length,
        selectedCompletionIdx,
    ]);

    const applyCompletionItem = useCallback(
        (item: EldritchCompletionItem) => {
            if (!completionResult) return;
            const before = effectiveRcCode.slice(0, completionResult.startOffset);
            const after = effectiveRcCode.slice(completionResult.endOffset);
            const nextCode = before + item.insertText + after;
            const nextCursor = completionResult.startOffset + item.insertText.length;
            commitCodeChange(nextCode, nextCursor, nextCursor);
            setDismissedCompletionAt(nextCursor);
            textareaRef.current?.focus();
        },
        [effectiveRcCode, completionResult, commitCodeChange]
    );

    const applyQuickFix = useCallback(
        (diag: EldritchDiagnostic) => {
            if (!diag.replacement) return;
            const lines = effectiveRcCode.split("\n");
            const targetLine = lines[diag.replacement.line];
            if (targetLine === undefined) return;
            lines[diag.replacement.line] =
                targetLine.slice(0, diag.replacement.startCol) +
                diag.replacement.newText +
                targetLine.slice(diag.replacement.endCol);
            const nextCode = lines.join("\n");
            commitCodeChange(nextCode, nextCode.length, nextCode.length);
        },
        [effectiveRcCode, commitCodeChange]
    );

    const toggleCommentSelection = useCallback(
        (start: number, end: number) => {
            const lineStart = effectiveRcCode.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
            const lineEndIdx = effectiveRcCode.indexOf("\n", end);
            const lineEnd = lineEndIdx === -1 ? effectiveRcCode.length : lineEndIdx;
            const block = effectiveRcCode.slice(lineStart, lineEnd);
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
            const nextCode =
                effectiveRcCode.slice(0, lineStart) + newBlock + effectiveRcCode.slice(lineEnd);
            const delta = newBlock.length - block.length;
            commitCodeChange(nextCode, start, Math.max(start, end + delta));
        },
        [effectiveRcCode, commitCodeChange]
    );

    const handleSelectRcTome = (tome: ShellTomeItem) => {
        const formatted = formatTomeForComposer(tome);
        setIsEditingScript(true);
        setTomePickerOpen(false);
        commitCodeChange(formatted, formatted.length, formatted.length);
        handleSetRcEnabled(true);
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    const handleSetupShellAliases = () => {
        if (!shellAliasesTome) return;
        const formatted = formatTomeForComposer(shellAliasesTome);
        setIsEditingScript(true);
        setTomePickerOpen(false);
        commitCodeChange(formatted, formatted.length, formatted.length);
        handleSetRcEnabled(true);
        (onToggle ?? onClose)?.();
    };

    const handleOpenCustomScript = () => {
        setIsEditingScript(true);
        setTomePickerOpen(false);
        commitCodeChange("", 0, 0);
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    if (!isOpen) {
        return null;
    }

    const lines = effectiveRcCode.split("\n");
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
        const lineIdx = Math.max(0, Math.floor(offsetY / 20));
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

        if (isMod && e.key === "/") {
            e.preventDefault();
            toggleCommentSelection(start, end);
            return;
        }

        if (e.ctrlKey && e.code === "Space") {
            e.preventDefault();
            setDismissedCompletionAt(null);
            setCursorOffset(end);
            return;
        }

        if (activeCompletions.length > 0 && start === end) {
            const visibleCount = Math.min(8, activeCompletions.length);
            if (
                (e.key === "Tab" && !e.shiftKey) ||
                (e.key === "Enter" && !isMod && !e.shiftKey)
            ) {
                e.preventDefault();
                applyCompletionItem(activeCompletions[selectedCompletionIdx] || activeCompletions[0]);
                return;
            }
            if (
                !isMod &&
                !e.altKey &&
                !e.shiftKey &&
                (e.key === "ArrowRight" || e.key === "ArrowDown")
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
            if (e.key === "Escape") {
                e.preventDefault();
                setDismissedCompletionAt(cursorOffset);
                return;
            }
        }

        if (e.key === "Tab") {
            e.preventDefault();

            if (start !== end || e.shiftKey) {
                const lineStart = effectiveRcCode.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
                const lineEndIdx = effectiveRcCode.indexOf("\n", end);
                const lineEnd = lineEndIdx === -1 ? effectiveRcCode.length : lineEndIdx;
                const block = effectiveRcCode.slice(lineStart, lineEnd);
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
                        effectiveRcCode.slice(0, lineStart) +
                        updatedLines.join("\n") +
                        effectiveRcCode.slice(lineEnd);
                    commitCodeChange(
                        nextCode,
                        Math.max(lineStart, start - removedFirst),
                        Math.max(lineStart, end - totalRemoved)
                    );
                } else {
                    const updatedLines = blockLines.map((line) => `    ${line}`);
                    const nextCode =
                        effectiveRcCode.slice(0, lineStart) +
                        updatedLines.join("\n") +
                        effectiveRcCode.slice(lineEnd);
                    commitCodeChange(nextCode, start + 4, end + 4 * blockLines.length);
                }
                return;
            }

            const nextCode = effectiveRcCode.slice(0, start) + "    " + effectiveRcCode.slice(end);
            commitCodeChange(nextCode, start + 4, start + 4);
            return;
        }

        if (e.key === "Enter" && !isMod && !e.shiftKey) {
            e.preventDefault();
            const lineStart = effectiveRcCode.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
            const currentLineBeforeCursor = effectiveRcCode.slice(lineStart, start);
            const indentMatch = currentLineBeforeCursor.match(/^(\s*)/);
            const baseIndent = indentMatch ? indentMatch[1] : "";
            const trimmedBefore = currentLineBeforeCursor.replace(/#.*$/, "").trimEnd();
            const extraIndent = trimmedBefore.endsWith(":") ? "    " : "";
            const insertion = `\n${baseIndent}${extraIndent}`;
            const nextCode = effectiveRcCode.slice(0, start) + insertion + effectiveRcCode.slice(end);
            const nextPos = start + insertion.length;
            commitCodeChange(nextCode, nextPos, nextPos);
            return;
        }

        if (e.key === "Backspace" && start === end && start > 0) {
            const lineStart = effectiveRcCode.lastIndexOf("\n", start - 1) + 1;
            const beforeOnLine = effectiveRcCode.slice(lineStart, start);
            if (
                beforeOnLine.length >= 4 &&
                /^ +$/.test(beforeOnLine) &&
                beforeOnLine.length % 4 === 0
            ) {
                e.preventDefault();
                const nextCode = effectiveRcCode.slice(0, start - 4) + effectiveRcCode.slice(start);
                commitCodeChange(nextCode, start - 4, start - 4);
                return;
            }
        }
    };

    const hoveredLineDiags =
        hoveredDiagLine !== null ? diagnosticsByLine.get(hoveredDiagLine) || null : null;

    const renderRcTomePopover = () => (
        <div
            data-testid="session-settings-tome-picker-popover"
            role="dialog"
            aria-label="Search and load RC tome"
            className="absolute right-0 top-full mt-1.5 w-80 max-w-[calc(100vw-32px)] rounded-lg bg-[#161a26] border border-[#2f364a] shadow-2xl z-30 overflow-hidden"
        >
            <div className="p-2 border-b border-[#262b38]">
                <div className="relative">
                    <Search className="w-3.5 h-3.5 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
                    <input
                        ref={tomeSearchInputRef}
                        type="text"
                        aria-label="Search RC tomes"
                        value={tomeSearchQuery}
                        onChange={(e) => setTomeSearchQuery(e.target.value)}
                        placeholder="Search RC tomes..."
                        className="w-full pl-8 pr-3 py-1.5 rounded-md bg-[#0f1117] border border-[#2a3040] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
                    />
                </div>
            </div>

            <div className="max-h-56 overflow-y-auto shell-scrollbar divide-y divide-[#222736]">
                {filteredRcTomes.length === 0 ? (
                    <div className="p-3 text-center text-xs text-gray-500">
                        No matching RC tomes found.
                    </div>
                ) : (
                    filteredRcTomes.map((tome) => (
                        <button
                            key={tome.id || tome.name}
                            type="button"
                            data-testid="session-settings-tome-option"
                            onClick={() => handleSelectRcTome(tome)}
                            className="w-full text-left px-3 py-2 hover:bg-[#202637] transition-colors space-y-0.5"
                        >
                            <div className="flex items-center justify-between gap-2">
                                <span className="text-xs font-semibold text-gray-200 truncate">
                                    {tome.name}
                                </span>
                                {tome.tactic && (
                                    <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-gray-400 shrink-0">
                                        {tome.tactic}
                                    </span>
                                )}
                            </div>
                            {tome.description && (
                                <p className="text-[11px] text-gray-400 line-clamp-1">
                                    {tome.description}
                                </p>
                            )}
                        </button>
                    ))
                )}
            </div>
        </div>
    );

    return (
        <div
            data-testid="session-settings-drawer"
            data-tour="shell-session-settings"
            className="flex flex-col border-t border-[#262b38] bg-[#141822] relative"
        >
            {/* Top Vertical Resize Drag Handle */}
            <div
                data-testid="session-settings-resize-handle"
                role="separator"
                aria-orientation="horizontal"
                aria-label="Resize session settings"
                tabIndex={0}
                onMouseDown={handleResizeMouseDown}
                onKeyDown={handleResizeKeyDown}
                title="Drag to resize height"
                className="h-1.5 w-full bg-[#181c28] hover:bg-white/10 cursor-ns-resize flex items-center justify-center transition-colors group"
            >
                <GripHorizontal className="w-3 h-3 text-gray-600 group-hover:text-gray-300" />
            </div>

            {/* SessionSettings Toolbar Header */}
            <div className="flex items-center justify-between gap-2 px-3 pb-1.5 pt-0.5 bg-[#181c28] border-b border-[#262b38]">
                <div className="flex items-center gap-2.5">
                    <span
                        className="inline-flex items-center justify-center p-1 text-gray-400"
                        title="Session Settings (RC)"
                    >
                        <Settings className="w-3.5 h-3.5" />
                    </span>

                    {/* Radio Switch Toggle to enable/disable RC */}
                    <button
                        type="button"
                        role="switch"
                        aria-checked={effectiveRcEnabled}
                        aria-label="Auto-run RC script on new shells"
                        data-testid="session-settings-rc-toggle"
                        onClick={handleToggleRcEnabled}
                        title={
                            effectiveRcEnabled
                                ? "RC enabled (click to disable)"
                                : "RC disabled (click to enable)"
                        }
                        className={`relative inline-flex h-4 w-7 shrink-0 cursor-pointer rounded-full border transition-colors ${
                            effectiveRcEnabled
                                ? "bg-purple-600 border-purple-500"
                                : "bg-[#11131a] border-[#2f364a]"
                        }`}
                    >
                        <span
                            className={`pointer-events-none inline-block h-3 w-3 transform rounded-full transition-transform mt-0.25 ${
                                effectiveRcEnabled
                                    ? "translate-x-3 bg-white"
                                    : "translate-x-0.5 bg-gray-400"
                            }`}
                        />
                    </button>

                    {isEditingScript && (
                        <button
                            type="button"
                            data-testid="rc-eldritch-lsp-status"
                            onClick={() => setShowDiagnosticsPanel((prev) => !prev)}
                            className={`inline-flex items-center gap-1 rounded-md text-[11px] font-mono transition-colors ${
                                errorCount > 0
                                    ? "px-2 py-0.5 bg-red-500/15 text-red-300 hover:bg-red-500/25"
                                    : warningCount > 0
                                      ? "px-2 py-0.5 bg-amber-500/15 text-amber-200 hover:bg-amber-500/25"
                                      : "p-1.5 text-emerald-400 hover:bg-[#272e3f]"
                            }`}
                            title="eldritch-lsp live diagnostics"
                        >
                            {errorCount > 0 ? (
                                <>
                                    <AlertCircle className="w-3.5 h-3.5 text-red-400" />
                                    <span>
                                        {errorCount} {errorCount === 1 ? "error" : "errors"}
                                    </span>
                                </>
                            ) : warningCount > 0 ? (
                                <>
                                    <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                                    <span>
                                        {warningCount} {warningCount === 1 ? "warning" : "warnings"}
                                    </span>
                                </>
                            ) : (
                                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                            )}
                        </button>
                    )}
                </div>

                <div className="flex items-center gap-1.5">
                    <div ref={tomePickerRef} className="relative">
                        <button
                            type="button"
                            data-testid="session-settings-tome-picker-button"
                            aria-label="Load RC tome"
                            aria-expanded={tomePickerOpen}
                            title="Load RC Tome"
                            onClick={() => setTomePickerOpen((prev) => !prev)}
                            className="inline-flex items-center justify-center p-1.5 rounded-md text-gray-300 hover:text-white hover:bg-[#272e3f] transition-colors"
                        >
                            <BookOpenIcon className="w-3.5 h-3.5" />
                        </button>
                        {tomePickerOpen && renderRcTomePopover()}
                    </div>

                    <button
                        type="button"
                        onClick={onToggle ?? onClose}
                        aria-label="Close session settings"
                        title="Close session settings"
                        className="p-1.5 rounded-md text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors"
                    >
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Body: Initial RC Tome / Custom Chooser vs Script Editor */}
            {!isEditingScript ? (
                <div
                    data-testid="session-settings-initial-chooser"
                    className="flex flex-col items-center justify-center gap-2.5 bg-[#0f1118] px-4 text-xs text-gray-300"
                    style={{ height: `${editorHeight}px` }}
                >
                    <div className="flex items-center gap-2.5">
                        <button
                            ref={loadTomeButtonRef}
                            type="button"
                            data-testid="session-settings-load-tome-button"
                            aria-label="RC Tome"
                            aria-expanded={tomePickerOpen}
                            onClick={() => setTomePickerOpen((prev) => !prev)}
                            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-[#181c28] hover:bg-[#222736] border border-[#2f364a] text-xs text-gray-200 font-medium transition-colors"
                        >
                            <BookOpenIcon className="w-3.5 h-3.5 text-gray-400" />
                            <span>RC Tome</span>
                        </button>

                        {shellAliasesTome && (
                            <button
                                type="button"
                                data-tour="shell-setup-aliases"
                                data-testid="session-settings-shell-aliases-setup"
                                onClick={handleSetupShellAliases}
                                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-purple-600 hover:bg-purple-500 border border-purple-500 text-xs text-white font-medium transition-colors"
                            >
                                <span>Set Default Aliases</span>
                                <ArrowRight className="w-3.5 h-3.5" />
                            </button>
                        )}
                    </div>

                    <button
                        type="button"
                        data-testid="session-settings-custom-button"
                        onClick={handleOpenCustomScript}
                        className="text-xs text-gray-400 hover:text-gray-200 transition-colors"
                    >
                        or custom
                    </button>
                </div>
            ) : (
                <div
                    onMouseLeave={() => setHoveredDiagLine(null)}
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
                                    onMouseEnter={() => {
                                        if (lineDiags && lineDiags.length > 0) {
                                            setHoveredDiagLine(idx);
                                        }
                                    }}
                                    className="flex items-center justify-end gap-1 h-5"
                                    title={lineDiags?.map((d) => d.message).join("\n")}
                                >
                                    {hasErr ? (
                                        <span className="w-1.5 h-1.5 rounded-full bg-red-400 inline-block" />
                                    ) : hasWarn ? (
                                        <span className="w-1.5 h-1.5 rounded-full bg-amber-400 inline-block" />
                                    ) : null}
                                    <span
                                        className={
                                            hasErr
                                                ? "text-red-400 font-semibold"
                                                : hasWarn
                                                  ? "text-amber-300"
                                                  : ""
                                        }
                                    >
                                        {idx + 1}
                                    </span>
                                </div>
                            );
                        })}
                    </div>

                    {/* Synchronized Syntax Highlighting Overlay + Inline Ghost Completion + Textarea */}
                    <div className="relative flex-1 h-full overflow-hidden">
                        <pre
                            ref={highlightPreRef}
                            aria-hidden="true"
                            data-testid="session-settings-syntax-highlight"
                            className="pointer-events-none absolute inset-0 m-0 py-2 px-3 font-mono text-xs leading-5 whitespace-pre overflow-hidden"
                        >
                            {lines.map((line, lineIdx) => {
                                const lineDiags = diagnosticsByLine.get(lineIdx);
                                const hasErr = lineDiags?.some((d) => d.severity === "error");
                                const hasWarn = lineDiags?.some((d) => d.severity === "warning");
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
                                        {effectiveRcCode.length === 0 && lineIdx === 0 ? (
                                            <span className="text-gray-500 italic">
                                                # Write or paste Eldritch code...
                                            </span>
                                        ) : beforeTokens.length === 0 && !showGhostOnLine ? (
                                            "\u200B"
                                        ) : (
                                            <>
                                                {beforeTokens.map((tok, tokIdx) => (
                                                    <span
                                                        key={`b-${tokIdx}`}
                                                        className={TOKEN_COLOR_CLASSES[tok.type]}
                                                    >
                                                        {tok.text}
                                                    </span>
                                                ))}
                                                {showGhostOnLine && (
                                                    <span className="text-gray-500/80 italic">
                                                        {inlineGhostInfo.suffix}
                                                    </span>
                                                )}
                                                {afterTokens.map((tok, tokIdx) => (
                                                    <span
                                                        key={`a-${tokIdx}`}
                                                        className={TOKEN_COLOR_CLASSES[tok.type]}
                                                    >
                                                        {tok.text}
                                                    </span>
                                                ))}
                                            </>
                                        )}
                                    </div>
                                );
                            })}
                        </pre>

                        <textarea
                            ref={textareaRef}
                            aria-label="Shared Eldritch RC script editor"
                            value={effectiveRcCode}
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
                        />

                        {/* Hover Lint Popover on Errored Code */}
                        {hoveredDiagLine !== null &&
                            hoveredLineDiags &&
                            hoveredLineDiags.length > 0 && (
                                <div
                                    data-testid="session-settings-lint-popover"
                                    role="tooltip"
                                    style={{
                                        top: `${Math.min(
                                            Math.max(
                                                4,
                                                (hoveredDiagLine + 1) * 20 + 6 - editorScrollTop
                                            ),
                                            Math.max(4, editorHeight - 56)
                                        )}px`,
                                        left: "12px",
                                    }}
                                    className="absolute z-30 max-w-lg rounded-md bg-[#161a26] border border-[#2f364a] px-3 py-2 text-xs shadow-xl space-y-1.5"
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
                                                        eldritch-lsp({diag.code}) • Ln{" "}
                                                        {diag.line + 1}, Col {diag.startCol + 1}
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
                                                    className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-[#1e222d] hover:bg-[#272d3d] border border-[#2f364a] text-gray-200 text-[11px] font-mono shrink-0 transition-colors"
                                                >
                                                    <Wand2 className="w-3 h-3 text-gray-400" />
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
                            data-testid="session-settings-diagnostics-list"
                            className="absolute bottom-0 left-0 right-0 z-20 max-h-28 overflow-y-auto divide-y divide-[#222736] bg-[#131722]/95 backdrop-blur-sm border-t border-[#262b38] text-xs shadow-lg"
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
                                        <span className="font-mono text-[11px] text-gray-400 shrink-0">
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
                                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-[#1e222d] hover:bg-[#272d3d] border border-[#2f364a] text-gray-200 text-[11px] font-mono shrink-0 transition-colors"
                                        >
                                            <Wand2 className="w-3 h-3 text-gray-400" />
                                            {diag.replacement.label}
                                        </button>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export { DEFAULT_RC_ALIASES_SCRIPT };
export default SessionSettings;

