import React, { useState, useRef, useEffect, useMemo } from "react";
import { BookOpenIcon } from "@heroicons/react/24/outline";
import { Code2, Play, Plus, Search, X } from "lucide-react";
import { analyzeEldritchCode } from "../utils/eldritchLsp";
import { ShellTomeItem } from "../hooks/useShellData";
import EldritchCodeEditor, {
    EldritchLspStatusButton,
    useResizableEditorHeight,
} from "./EldritchCodeEditor";
import SaveTomeModal from "./SaveTomeModal";

export const RC_SCRIPT_STORAGE_KEY = "tavern_shell_rc_script_v1";
export const RC_ENABLED_STORAGE_KEY = "tavern_shell_rc_enabled_v1";
export const RC_EXECUTED_SHELLS_STORAGE_KEY = "tavern_shell_rc_executed_shells_v1";
export const SHELL_FONT_SIZE_STORAGE_KEY = "tavern_shell_font_size_v1";

export const MIN_SHELL_FONT_SIZE = 6;
export const MAX_SHELL_FONT_SIZE = 28;
export const DEFAULT_SHELL_FONT_SIZE = 13;

export function clampShellFontSize(size: number): number {
    if (!Number.isFinite(size)) {
        return DEFAULT_SHELL_FONT_SIZE;
    }
    return Math.min(MAX_SHELL_FONT_SIZE, Math.max(MIN_SHELL_FONT_SIZE, Math.round(size)));
}

export function loadShellFontSize(): number {
    try {
        const stored = localStorage.getItem(SHELL_FONT_SIZE_STORAGE_KEY);
        if (stored !== null) {
            const parsed = Number(stored);
            if (Number.isFinite(parsed)) {
                return clampShellFontSize(parsed);
            }
        }
    } catch {
        // Ignore storage error
    }
    return DEFAULT_SHELL_FONT_SIZE;
}

export function saveShellFontSize(size: number): void {
    try {
        localStorage.setItem(SHELL_FONT_SIZE_STORAGE_KEY, String(clampShellFontSize(size)));
    } catch {
        // Ignore storage error
    }
}

export const DEFAULT_RC_ALIASES_SCRIPT = [
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
].join("\n");

export function loadRcScript(): string {
    try {
        const stored = localStorage.getItem(RC_SCRIPT_STORAGE_KEY);
        if (stored !== null) {
            return stored;
        }
    } catch {
        // Ignore storage error
    }
    return "";
}

export function saveRcScript(script: string): void {
    try {
        localStorage.setItem(RC_SCRIPT_STORAGE_KEY, script);
    } catch {
        // Ignore storage error
    }
}

export function loadRcEnabled(): boolean {
    try {
        return localStorage.getItem(RC_ENABLED_STORAGE_KEY) === "true";
    } catch {
        return false;
    }
}

export function saveRcEnabled(enabled: boolean): void {
    try {
        localStorage.setItem(RC_ENABLED_STORAGE_KEY, enabled ? "true" : "false");
    } catch {
        // Ignore storage error
    }
}

export function hasShellExecutedRc(shellId: string): boolean {
    if (!shellId) return false;
    try {
        const raw = localStorage.getItem(RC_EXECUTED_SHELLS_STORAGE_KEY);
        if (!raw) return false;
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) && parsed.includes(String(shellId));
    } catch {
        return false;
    }
}

export function markShellExecutedRc(shellId: string): void {
    if (!shellId) return;
    try {
        const raw = localStorage.getItem(RC_EXECUTED_SHELLS_STORAGE_KEY);
        const parsed = raw ? JSON.parse(raw) : [];
        const list = Array.isArray(parsed) ? parsed : [];
        if (!list.includes(String(shellId))) {
            list.push(String(shellId));
            localStorage.setItem(
                RC_EXECUTED_SHELLS_STORAGE_KEY,
                JSON.stringify(list.slice(-200))
            );
        }
    } catch {
        // Ignore storage error
    }
}

export const FALLBACK_ELDRITCH_TOMES: ShellTomeItem[] = [
    {
        id: "tome-shell-aliases",
        name: "Shell Aliases",
        description:
            "Define common shell command aliases (ls, whoami, ps, pwd, cat, netstat) in the Eldritch REPL.",
        tactic: "SHELL_RC",
        supportModel: "FIRST_PARTY",
        eldritch: DEFAULT_RC_ALIASES_SCRIPT,
    },
    {
        id: "tome-file-list",
        name: "List files",
        description: "List the files and directories found at the path.",
        tactic: "RECON",
        paramDefs: JSON.stringify([
            { name: "path", type: "string", label: "File path", placeholder: "/etc/open*" },
        ]),
        eldritch: [
            "def file_list(path):",
            "    res = file.list(path)",
            "    for f in res:",
            '        print(f"{f[\'permissions\']}\\t{f[\'owner\']}\\t{f[\'size\']}\\t{f[\'absolute_path\']}")',
            "",
            'file_list(input_params["path"])',
        ].join("\n"),
    },
    {
        id: "tome-process-list",
        name: "Process list",
        description: "List running processes on the target system.",
        tactic: "RECON",
        eldritch: [
            "for p in process.list():",
            '    print(f"{p[\'pid\']}\\t{p[\'username\']}\\t{p[\'name\']}\\t{p[\'command\']}")',
        ].join("\n"),
    },
    {
        id: "tome-sys-info",
        name: "System info",
        description: "Gather host OS, user identity, and network interface details.",
        tactic: "RECON",
        eldritch: [
            "pprint(sys.info())",
            "pprint(user.info())",
            "tprint(net.interfaces())",
        ].join("\n"),
    },
    {
        id: "tome-cat-file",
        name: "Cat file",
        description: "Read and print the contents of a file.",
        tactic: "COLLECTION",
        paramDefs: JSON.stringify([
            { name: "path", type: "string", label: "File path", placeholder: "/etc/passwd" },
        ]),
        eldritch: 'print(str(file.read(input_params["path"])))',
    },
    {
        id: "tome-arp-scan",
        name: "ARP scan",
        description: "Discover active hosts on the local subnet via ARP.",
        tactic: "DISCOVERY",
        paramDefs: JSON.stringify([
            { name: "target_cidrs", type: "string", label: "Target CIDR", placeholder: "192.168.1.0/24" },
        ]),
        eldritch: 'tprint(scan.arp([input_params["target_cidrs"]]))',
    },
];

export function formatTomeForComposer(tome: ShellTomeItem): string {
    const headerLines = [`# Tome: ${tome.name}`];
    if (tome.description) {
        headerLines.push(`# ${tome.description.replace(/\r?\n/g, " ").trim()}`);
    }

    let paramsBlock = "";
    if (tome.paramDefs && tome.eldritch.includes("input_params")) {
        try {
            const defs = JSON.parse(tome.paramDefs);
            if (Array.isArray(defs) && defs.length > 0) {
                const entries = defs.map((d: any) => {
                    const defaultVal = d.default ?? d.placeholder ?? "";
                    const comment = d.label || d.description ? `  # ${d.label || d.description}` : "";
                    return `    ${JSON.stringify(d.name)}: ${JSON.stringify(String(defaultVal))},${comment}`;
                });
                paramsBlock = `input_params = {\n${entries.join("\n")}\n}\n\n`;
            }
        } catch {
            // Ignore malformed paramDefs JSON
        }
    }

    return `${headerLines.join("\n")}\n${paramsBlock}${tome.eldritch.trim()}`;
}

interface MultiLineComposerProps {
    isOpen: boolean;
    onToggle: () => void;
    code: string;
    onChange: (nextCode: string) => void;
    onRunCode: (code: string) => void;
    onStageToPrompt?: (code: string) => void;
    disabled?: boolean;
    tomes?: ShellTomeItem[];
    defaultAuthor?: string;
    onTomeCreated?: (tome: ShellTomeItem) => void;
    rcCode?: string;
    onRcCodeChange?: (nextRcCode: string) => void;
    rcEnabled?: boolean;
    onRcEnabledChange?: (enabled: boolean) => void;
    fontSize?: number;
}

const MultiLineComposer: React.FC<MultiLineComposerProps> = ({
    isOpen,
    onToggle,
    code,
    onChange,
    onRunCode,
    disabled = false,
    tomes,
    defaultAuthor,
    onTomeCreated,
    fontSize,
}) => {
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const tomePickerRef = useRef<HTMLDivElement>(null);
    const tomeSearchInputRef = useRef<HTMLInputElement>(null);

    const [tomePickerOpen, setTomePickerOpen] = useState<boolean>(false);
    const [tomeSearchQuery, setTomeSearchQuery] = useState<string>("");
    const [saveTomeModalOpen, setSaveTomeModalOpen] = useState<boolean>(false);
    const [localSavedTomes, setLocalSavedTomes] = useState<ShellTomeItem[]>([]);
    const [showDiagnosticsPanel, setShowDiagnosticsPanel] = useState<boolean>(true);

    const { editorHeight, handleResizeMouseDown, handleResizeKeyDown } =
        useResizableEditorHeight();

    const allKnownTomes = useMemo(() => {
        const base = tomes && tomes.length > 0 ? tomes : FALLBACK_ELDRITCH_TOMES;
        const byName = new Map<string, ShellTomeItem>();
        for (const t of base) {
            byName.set(t.name.toLowerCase(), t);
        }
        for (const t of localSavedTomes) {
            byName.set(t.name.toLowerCase(), t);
        }
        return Array.from(byName.values());
    }, [tomes, localSavedTomes]);

    const availableTomes = useMemo(() => {
        return allKnownTomes.filter((t) => t.tactic?.toUpperCase() !== "SHELL_RC");
    }, [allKnownTomes]);

    const filteredTomes = useMemo(() => {
        const q = tomeSearchQuery.trim().toLowerCase();
        if (!q) return availableTomes;
        return availableTomes.filter(
            (t) =>
                t.name.toLowerCase().includes(q) ||
                (t.description && t.description.toLowerCase().includes(q)) ||
                (t.tactic && t.tactic.toLowerCase().includes(q)) ||
                t.eldritch.toLowerCase().includes(q)
        );
    }, [availableTomes, tomeSearchQuery]);

    useEffect(() => {
        if (!tomePickerOpen) return;
        const handleClickOutside = (e: MouseEvent) => {
            if (tomePickerRef.current && !tomePickerRef.current.contains(e.target as Node)) {
                setTomePickerOpen(false);
            }
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, [tomePickerOpen]);

    useEffect(() => {
        if (tomePickerOpen) {
            setTimeout(() => tomeSearchInputRef.current?.focus(), 10);
        } else {
            setTomeSearchQuery("");
        }
    }, [tomePickerOpen]);

    const diagnostics = useMemo(() => analyzeEldritchCode(code), [code]);

    const handleSelectTome = (tome: ShellTomeItem) => {
        const formatted = formatTomeForComposer(tome);
        onChange(formatted);
        setTomePickerOpen(false);
        setTimeout(() => textareaRef.current?.focus(), 20);
    };

    if (!isOpen) {
        return null;
    }

    return (
        <div
            data-testid="multiline-composer"
            data-tour="shell-composer"
            className="flex flex-col border-t border-gray-800 bg-gray-900 overflow-hidden relative"
        >
            {/* Top Vertical Resize Drag Handle (Overlaid on Top Border for Compact Header) */}
            <div
                data-testid="composer-resize-handle"
                role="separator"
                aria-orientation="horizontal"
                aria-label="Resize composer"
                tabIndex={0}
                onMouseDown={handleResizeMouseDown}
                onKeyDown={handleResizeKeyDown}
                title="Drag to resize composer height"
                className="absolute inset-x-0 top-0 h-1 bg-transparent hover:bg-purple-500/50 cursor-ns-resize z-20 transition-colors"
            />

            {/* Composer Toolbar Header */}
            <div className="flex items-center justify-between gap-2 px-3 py-1 bg-gray-900 border-b border-gray-800 text-xs text-gray-400 select-none">
                <div className="flex items-center gap-1.5">
                    <span
                        className="inline-flex items-center justify-center p-1 text-purple-400"
                        title="Eldritch Composer"
                    >
                        <Code2 className="w-4 h-4" />
                    </span>

                    <EldritchLspStatusButton
                        diagnostics={diagnostics}
                        onClick={() => setShowDiagnosticsPanel((prev) => !prev)}
                        testId="eldritch-lsp-status"
                        includeWarningSubcountOnError={true}
                    />
                </div>

                <div className="flex items-center gap-1.5">
                    {/* Searchable Eldritch Tome Picker */}
                    <div ref={tomePickerRef} className="relative flex items-center">
                        <button
                            type="button"
                            data-tour="shell-composer-tomes"
                            data-testid="composer-tome-picker-button"
                            aria-label="Load Eldritch tome"
                            aria-expanded={tomePickerOpen}
                            title="Load Tome"
                            onClick={() => setTomePickerOpen((prev) => !prev)}
                            className="inline-flex items-center justify-center p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                        >
                            <BookOpenIcon className="w-4 h-4" />
                        </button>

                        {tomePickerOpen && (
                            <div
                                data-testid="composer-tome-picker-popover"
                                role="dialog"
                                aria-label="Search and load Eldritch tome"
                                className="absolute right-0 top-full mt-1.5 w-80 max-w-[calc(100vw-32px)] rounded-lg bg-gray-900 border border-gray-700 shadow-2xl z-30 overflow-hidden"
                            >
                                <div className="p-2 border-b border-gray-800">
                                    <div className="relative">
                                        <Search className="w-4 h-4 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                                        <input
                                            ref={tomeSearchInputRef}
                                            type="text"
                                            aria-label="Search Eldritch tomes"
                                            value={tomeSearchQuery}
                                            onChange={(e) => setTomeSearchQuery(e.target.value)}
                                            placeholder="Search tomes by name, tactic, or description..."
                                            className="w-full pl-8 pr-3 py-1.5 rounded-md bg-gray-800 border border-gray-700 text-xs text-gray-200 placeholder-gray-400 focus:outline-none focus:border-purple-500"
                                        />
                                    </div>
                                </div>

                                <div className="max-h-56 overflow-y-auto shell-scrollbar divide-y divide-gray-800">
                                    {filteredTomes.length === 0 ? (
                                        <div className="p-3 text-center text-xs text-gray-500">
                                            No matching Eldritch tomes found.
                                        </div>
                                    ) : (
                                        filteredTomes.map((tome) => (
                                            <button
                                                key={tome.id || tome.name}
                                                type="button"
                                                data-testid="composer-tome-option"
                                                onClick={() => handleSelectTome(tome)}
                                                className="w-full text-left px-3 py-2 hover:bg-gray-800 transition-colors space-y-0.5"
                                            >
                                                <div className="flex items-center justify-between gap-2">
                                                    <span className="text-xs font-semibold text-gray-200 truncate">
                                                        {tome.name}
                                                    </span>
                                                    {tome.tactic && (
                                                        <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-gray-800 text-gray-400 shrink-0">
                                                            {tome.tactic}
                                                        </span>
                                                    )}
                                                </div>
                                                {tome.description && (
                                                    <p className="text-xs text-gray-400 line-clamp-1">
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

                    {/* Save Tome Button (Tome icon with Plus overlayed, aligned next to existing Tome button) */}
                    <button
                        type="button"
                        data-testid="composer-save-tome-button"
                        aria-label="Save Tome"
                        title="Save Tome"
                        onClick={() => {
                            setTomePickerOpen(false);
                            setSaveTomeModalOpen(true);
                        }}
                        className="inline-flex items-center justify-center p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                    >
                        <span className="relative inline-flex items-center justify-center">
                            <BookOpenIcon className="w-4 h-4" />
                            <Plus
                                data-testid="composer-save-tome-plus-icon"
                                className="w-2.5 h-2.5 absolute -top-1 -right-1 bg-gray-900 rounded-full stroke-[2.5]"
                            />
                        </span>
                    </button>

                    <button
                        type="button"
                        disabled={!code.trim() || disabled}
                        onClick={() => onRunCode(code)}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-purple-600 hover:bg-purple-500 disabled:opacity-40 disabled:cursor-not-allowed text-xs font-semibold text-white transition-colors shadow-sm"
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
                        className="inline-flex items-center justify-center p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
            </div>

            <EldritchCodeEditor
                code={code}
                onChange={onChange}
                onRunCode={onRunCode}
                disabled={disabled}
                editorHeight={editorHeight}
                fontSize={fontSize}
                ariaLabel="Multi-line Eldritch script editor"
                testIdPrefix="composer"
                showDiagnosticsPanel={showDiagnosticsPanel}
                autoFocus={isOpen}
                textareaRefExternal={textareaRef}
            />

            {saveTomeModalOpen && (
                <SaveTomeModal
                    isOpen={saveTomeModalOpen}
                    onClose={() => setSaveTomeModalOpen(false)}
                    composerCode={code}
                    existingTomes={allKnownTomes}
                    defaultAuthor={defaultAuthor}
                    onTomeCreated={(createdTome) => {
                        setLocalSavedTomes((prev) => [...prev, createdTome]);
                        onTomeCreated?.(createdTome);
                    }}
                />
            )}
        </div>
    );
};

export default MultiLineComposer;
