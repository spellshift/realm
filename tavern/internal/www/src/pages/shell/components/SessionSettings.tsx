import React, { useState, useRef, useEffect, useMemo, useCallback } from "react";
import { BookOpenIcon } from "@heroicons/react/24/outline";
import {
    AArrowDown,
    AArrowUp,
    ArrowRight,
    GripHorizontal,
    RotateCcw,
    Search,
    Settings,
    X,
} from "lucide-react";
import { analyzeEldritchCode } from "../utils/eldritchLsp";
import { ShellTomeItem } from "../hooks/useShellData";
import EldritchCodeEditor, {
    EldritchLspStatusButton,
    useResizableEditorHeight,
} from "./EldritchCodeEditor";
import {
    DEFAULT_RC_ALIASES_SCRIPT,
    DEFAULT_SHELL_FONT_SIZE,
    FALLBACK_ELDRITCH_TOMES,
    MAX_SHELL_FONT_SIZE,
    MIN_SHELL_FONT_SIZE,
    SHELL_FONT_SIZE_STORAGE_KEY,
    clampShellFontSize,
    formatTomeForComposer,
    loadRcEnabled,
    loadRcScript,
    loadShellFontSize,
    saveRcEnabled,
    saveRcScript,
    saveShellFontSize,
} from "./MultiLineComposer";

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
    fontSize?: number;
    onFontSizeChange?: (nextFontSize: number) => void;
}

interface SessionGeneralSettingsPanelProps {
    fontSize: number;
    onDecreaseFontSize: () => void;
    onIncreaseFontSize: () => void;
    onResetFontSize: () => void;
}

const SessionGeneralSettingsPanel: React.FC<SessionGeneralSettingsPanelProps> = ({
    fontSize,
    onDecreaseFontSize,
    onIncreaseFontSize,
    onResetFontSize,
}) => (
    <div
        data-testid="session-settings-left-panel"
        className="w-44 shrink-0 border-r border-[#262b38] bg-[#12151e] p-2.5 flex flex-col gap-2 text-xs text-gray-300"
    >
        <div className="flex items-center justify-between">
            <span className="text-[11px] font-medium text-gray-400">Font Size</span>
            <span
                data-testid="session-settings-font-size-value"
                className="font-mono text-[11px] text-gray-300"
            >
                {fontSize}px
            </span>
        </div>

        <div className="flex items-center gap-1">
            <button
                type="button"
                data-testid="session-settings-font-decrease"
                aria-label="Decrease font size"
                title={`Decrease font size (${fontSize}px)`}
                disabled={fontSize <= MIN_SHELL_FONT_SIZE}
                onClick={onDecreaseFontSize}
                className="inline-flex items-center justify-center p-1.5 rounded-md bg-[#181c28] hover:bg-[#272e3f] border border-[#262b38] text-gray-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
                <AArrowDown className="w-3.5 h-3.5" />
            </button>

            <button
                type="button"
                data-testid="session-settings-font-increase"
                aria-label="Increase font size"
                title={`Increase font size (${fontSize}px)`}
                disabled={fontSize >= MAX_SHELL_FONT_SIZE}
                onClick={onIncreaseFontSize}
                className="inline-flex items-center justify-center p-1.5 rounded-md bg-[#181c28] hover:bg-[#272e3f] border border-[#262b38] text-gray-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
                <AArrowUp className="w-3.5 h-3.5" />
            </button>

            <button
                type="button"
                data-testid="session-settings-font-reset"
                aria-label="Reset font size"
                title={`Reset font size (${DEFAULT_SHELL_FONT_SIZE}px)`}
                disabled={fontSize === DEFAULT_SHELL_FONT_SIZE}
                onClick={onResetFontSize}
                className="inline-flex items-center justify-center gap-1 px-2 py-1 rounded-md bg-[#181c28] hover:bg-[#272e3f] border border-[#262b38] text-[11px] text-gray-300 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors ml-auto"
            >
                <RotateCcw className="w-3 h-3" />
                <span>Reset</span>
            </button>
        </div>
    </div>
);

interface RcInitialChooserProps {
    editorHeight: number;
    loadTomeButtonRef: React.RefObject<HTMLButtonElement>;
    tomePickerOpen: boolean;
    onToggleTomePicker: () => void;
    hasShellAliasesTome: boolean;
    onSetupShellAliases: () => void;
    onOpenCustomScript: () => void;
}

const RcInitialChooser: React.FC<RcInitialChooserProps> = ({
    editorHeight,
    loadTomeButtonRef,
    tomePickerOpen,
    onToggleTomePicker,
    hasShellAliasesTome,
    onSetupShellAliases,
    onOpenCustomScript,
}) => (
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
                onClick={onToggleTomePicker}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-[#181c28] hover:bg-[#222736] border border-[#2f364a] text-xs text-gray-200 font-medium transition-colors"
            >
                <BookOpenIcon className="w-3.5 h-3.5 text-gray-400" />
                <span>RC Tome</span>
            </button>

            {hasShellAliasesTome && (
                <button
                    type="button"
                    data-tour="shell-setup-aliases"
                    data-testid="session-settings-shell-aliases-setup"
                    onClick={onSetupShellAliases}
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
            onClick={onOpenCustomScript}
            className="text-xs text-gray-400 hover:text-gray-200 transition-colors"
        >
            or custom
        </button>
    </div>
);

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
    fontSize: controlledFontSize,
    onFontSizeChange,
}) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const tomePickerRef = useRef<HTMLDivElement>(null);
    const loadTomeButtonRef = useRef<HTMLButtonElement>(null);
    const tomeSearchInputRef = useRef<HTMLInputElement>(null);

    const [internalRcCode, setInternalRcCode] = useState<string>(
        () => controlledRcCode ?? loadRcScript()
    );
    const [internalRcEnabled, setInternalRcEnabled] = useState<boolean>(
        () => controlledRcEnabled ?? loadRcEnabled()
    );
    const [internalFontSize, setInternalFontSize] = useState<number>(() =>
        controlledFontSize !== undefined
            ? clampShellFontSize(controlledFontSize)
            : loadShellFontSize()
    );
    const [isEditingScript, setIsEditingScript] = useState<boolean>(
        () => (controlledRcCode ?? loadRcScript()).trim().length > 0
    );
    const [showDiagnosticsPanel, setShowDiagnosticsPanel] = useState<boolean>(true);
    const [tomePickerOpen, setTomePickerOpen] = useState<boolean>(false);
    const [tomeSearchQuery, setTomeSearchQuery] = useState<string>("");

    const { editorHeight, handleResizeMouseDown, handleResizeKeyDown } =
        useResizableEditorHeight();

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

    useEffect(() => {
        if (controlledFontSize !== undefined) {
            setInternalFontSize(clampShellFontSize(controlledFontSize));
        }
    }, [controlledFontSize]);

    const effectiveRcCode = internalRcCode;
    const effectiveRcEnabled = internalRcEnabled;
    const effectiveFontSize = internalFontSize;

    const handleFontSizeChange = useCallback(
        (nextSize: number) => {
            const clamped = clampShellFontSize(nextSize);
            setInternalFontSize(clamped);
            saveShellFontSize(clamped);
            onFontSizeChange?.(clamped);
        },
        [onFontSizeChange]
    );

    const handleDecreaseFontSize = useCallback(() => {
        handleFontSizeChange(effectiveFontSize - 1);
    }, [effectiveFontSize, handleFontSizeChange]);

    const handleIncreaseFontSize = useCallback(() => {
        handleFontSizeChange(effectiveFontSize + 1);
    }, [effectiveFontSize, handleFontSizeChange]);

    const handleResetFontSize = useCallback(() => {
        handleFontSizeChange(DEFAULT_SHELL_FONT_SIZE);
    }, [handleFontSizeChange]);

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

    const diagnostics = useMemo(
        () => analyzeEldritchCode(effectiveRcCode),
        [effectiveRcCode]
    );

    const handleSelectRcTome = (tome: ShellTomeItem) => {
        const formatted = formatTomeForComposer(tome);
        setIsEditingScript(true);
        setTomePickerOpen(false);
        handleRcCodeChange(formatted);
        handleSetRcEnabled(true);
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    const handleSetupShellAliases = () => {
        if (!shellAliasesTome) return;
        const formatted = formatTomeForComposer(shellAliasesTome);
        setIsEditingScript(true);
        setTomePickerOpen(false);
        handleRcCodeChange(formatted);
        handleSetRcEnabled(true);
        (onToggle ?? onClose)?.();
    };

    const handleOpenCustomScript = () => {
        setIsEditingScript(true);
        setTomePickerOpen(false);
        handleRcCodeChange("");
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    if (!isOpen) {
        return null;
    }

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
                        <EldritchLspStatusButton
                            diagnostics={diagnostics}
                            onClick={() => setShowDiagnosticsPanel((prev) => !prev)}
                            testId="rc-eldritch-lsp-status"
                        />
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

                        {tomePickerOpen && (
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
                        )}
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

            {/* Vertically Split Body: Left General Settings Panel + Right RC Script Pane */}
            <div className="flex min-w-0">
                <SessionGeneralSettingsPanel
                    fontSize={effectiveFontSize}
                    onDecreaseFontSize={handleDecreaseFontSize}
                    onIncreaseFontSize={handleIncreaseFontSize}
                    onResetFontSize={handleResetFontSize}
                />

                <div className="flex-1 min-w-0">
                    {!isEditingScript ? (
                        <RcInitialChooser
                            editorHeight={editorHeight}
                            loadTomeButtonRef={loadTomeButtonRef}
                            tomePickerOpen={tomePickerOpen}
                            onToggleTomePicker={() => setTomePickerOpen((prev) => !prev)}
                            hasShellAliasesTome={Boolean(shellAliasesTome)}
                            onSetupShellAliases={handleSetupShellAliases}
                            onOpenCustomScript={handleOpenCustomScript}
                        />
                    ) : (
                        <EldritchCodeEditor
                            code={effectiveRcCode}
                            onChange={handleRcCodeChange}
                            disabled={disabled}
                            editorHeight={editorHeight}
                            fontSize={effectiveFontSize}
                            ariaLabel="Shared Eldritch RC script editor"
                            testIdPrefix="session-settings"
                            showDiagnosticsPanel={showDiagnosticsPanel}
                            autoFocus={isOpen && isEditingScript}
                            textareaRefExternal={textareaRef}
                        />
                    )}
                </div>
            </div>
        </div>
    );
};

export {
    DEFAULT_RC_ALIASES_SCRIPT,
    DEFAULT_SHELL_FONT_SIZE,
    MAX_SHELL_FONT_SIZE,
    MIN_SHELL_FONT_SIZE,
    SHELL_FONT_SIZE_STORAGE_KEY,
    clampShellFontSize,
    loadShellFontSize,
    saveShellFontSize,
};
export default SessionSettings;
