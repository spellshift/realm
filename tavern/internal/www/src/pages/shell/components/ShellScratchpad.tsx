import React, { useState, useRef, useEffect, useCallback, useContext } from "react";
import { useMutation, useQuery } from "@apollo/client";
import { NotebookPen, RefreshCw, WifiOff, X } from "lucide-react";
import { AuthorizationContext } from "../../../context/AuthorizationContext";
import {
    CREATE_OPERATOR_NOTE_MUTATION,
    GET_OPERATOR_NOTES,
    UPDATE_OPERATOR_NOTE_MUTATION,
} from "../graphql";
import { useResizableEditorHeight } from "./EldritchCodeEditor";

export const SCRATCHPAD_STORAGE_KEY = "tavern_shell_scratchpad_v1";
export const SCRATCHPAD_SYNC_EVENT = "tavern_shell_scratchpad_sync";
const SAVE_DEBOUNCE_MS = 150;

export interface StoredScratchpadState {
    content: string;
    noteId: string | null;
    shellIds: string[];
    pendingShellIds: string[];
    pendingSync: boolean;
}

const DEFAULT_SCRATCHPAD_STATE: StoredScratchpadState = {
    content: "",
    noteId: null,
    shellIds: [],
    pendingShellIds: [],
    pendingSync: false,
};

export function loadScratchpadState(): StoredScratchpadState {
    try {
        const raw = localStorage.getItem(SCRATCHPAD_STORAGE_KEY);
        if (!raw) {
            return { ...DEFAULT_SCRATCHPAD_STATE };
        }
        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== "object") {
            return { ...DEFAULT_SCRATCHPAD_STATE };
        }
        return {
            content: typeof parsed.content === "string" ? parsed.content : "",
            noteId:
                parsed.noteId !== undefined && parsed.noteId !== null
                    ? String(parsed.noteId)
                    : null,
            shellIds: Array.isArray(parsed.shellIds)
                ? parsed.shellIds.map((id: any) => String(id)).filter(Boolean)
                : [],
            pendingShellIds: Array.isArray(parsed.pendingShellIds)
                ? parsed.pendingShellIds.map((id: any) => String(id)).filter(Boolean)
                : [],
            pendingSync: Boolean(parsed.pendingSync),
        };
    } catch {
        return { ...DEFAULT_SCRATCHPAD_STATE };
    }
}

export function saveScratchpadState(state: StoredScratchpadState): void {
    try {
        localStorage.setItem(SCRATCHPAD_STORAGE_KEY, JSON.stringify(state));
        if (typeof window !== "undefined") {
            window.dispatchEvent(
                new CustomEvent(SCRATCHPAD_SYNC_EVENT, { detail: state })
            );
        }
    } catch {
        // Ignore storage error
    }
}

function extractShellIdsFromNote(note: any): string[] {
    if (!note?.shells?.edges || !Array.isArray(note.shells.edges)) {
        return [];
    }
    const ids: string[] = [];
    for (const edge of note.shells.edges) {
        const id = edge?.node?.id;
        if (id !== undefined && id !== null && String(id) !== "") {
            const strId = String(id);
            if (!ids.includes(strId)) {
                ids.push(strId);
            }
        }
    }
    return ids;
}

function selectOperatorNoteForUser(edges: any[], currentUserId?: string): any | null {
    if (!Array.isArray(edges) || edges.length === 0) {
        return null;
    }
    if (currentUserId) {
        const matched = edges.find(
            (edge) =>
                edge?.node?.author?.id &&
                String(edge.node.author.id) === String(currentUserId)
        );
        if (matched?.node) {
            return matched.node;
        }
    }
    return edges[0]?.node ?? null;
}

export interface ShellScratchpadProps {
    isOpen: boolean;
    onToggle: () => void;
    shellId?: string;
    fontSize?: number;
}

const ShellScratchpad: React.FC<ShellScratchpadProps> = ({
    isOpen,
    onToggle,
    shellId,
    fontSize,
}) => {
    const authContext = useContext(AuthorizationContext);
    const currentUserId = authContext?.data?.me?.id;

    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const initialState = useRef<StoredScratchpadState>(loadScratchpadState());

    const [content, setContent] = useState<string>(() => initialState.current.content);
    const [isOffline, setIsOffline] = useState<boolean>(false);
    const [isSyncing, setIsSyncing] = useState<boolean>(false);

    const contentRef = useRef<string>(initialState.current.content);
    const noteIdRef = useRef<string | null>(initialState.current.noteId);
    const knownShellIdsRef = useRef<Set<string>>(new Set(initialState.current.shellIds));
    const pendingShellIdsRef = useRef<string[]>(initialState.current.pendingShellIds);
    const pendingSyncRef = useRef<boolean>(initialState.current.pendingSync);
    const isOfflineRef = useRef<boolean>(false);
    const inFlightRef = useRef<boolean>(false);
    const queuedSyncRef = useRef<boolean>(false);
    const initialLoadDoneRef = useRef<boolean>(false);
    const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const { editorHeight, handleResizeMouseDown, handleResizeKeyDown } =
        useResizableEditorHeight();

    const [createOperatorNoteMutation] = useMutation(CREATE_OPERATOR_NOTE_MUTATION);
    const [updateOperatorNoteMutation] = useMutation(UPDATE_OPERATOR_NOTE_MUTATION);

    const persistCurrentState = useCallback(() => {
        saveScratchpadState({
            content: contentRef.current,
            noteId: noteIdRef.current,
            shellIds: Array.from(knownShellIdsRef.current),
            pendingShellIds: [...pendingShellIdsRef.current],
            pendingSync: pendingSyncRef.current,
        });
    }, []);

    const syncNoteToBackend = useCallback(async (): Promise<boolean> => {
        if (!pendingSyncRef.current) {
            return true;
        }
        if (inFlightRef.current) {
            queuedSyncRef.current = true;
            return false;
        }

        inFlightRef.current = true;
        setIsSyncing(true);

        const contentToSave = contentRef.current;
        const targetNoteId = noteIdRef.current;
        const pendingShellsSnapshot = [...pendingShellIdsRef.current];

        try {
            let savedNote: any = null;

            if (targetNoteId) {
                const shellsToAdd = pendingShellsSnapshot.filter(
                    (id) => id && !knownShellIdsRef.current.has(String(id))
                );
                const input: Record<string, any> = {
                    content: contentToSave,
                };
                if (shellsToAdd.length > 0) {
                    input.addShellIDs = shellsToAdd;
                }
                const response = await updateOperatorNoteMutation({
                    variables: {
                        operatorNoteID: targetNoteId,
                        input,
                    },
                });
                savedNote = response.data?.updateOperatorNote;
                for (const sId of shellsToAdd) {
                    knownShellIdsRef.current.add(String(sId));
                }
            } else {
                const uniqueShells = Array.from(
                    new Set(pendingShellsSnapshot.filter(Boolean).map(String))
                );
                const input: Record<string, any> = {
                    content: contentToSave,
                };
                if (uniqueShells.length > 0) {
                    input.shellIDs = uniqueShells;
                }
                const response = await createOperatorNoteMutation({
                    variables: {
                        input,
                    },
                });
                savedNote = response.data?.createOperatorNote;
                for (const sId of uniqueShells) {
                    knownShellIdsRef.current.add(String(sId));
                }
            }

            if (savedNote?.id !== undefined && savedNote?.id !== null) {
                noteIdRef.current = String(savedNote.id);
            }
            for (const sId of extractShellIdsFromNote(savedNote)) {
                knownShellIdsRef.current.add(sId);
            }

            pendingShellIdsRef.current = pendingShellIdsRef.current.filter(
                (id) => !knownShellIdsRef.current.has(String(id))
            );

            const hasNewerEdits =
                contentRef.current !== contentToSave ||
                pendingShellIdsRef.current.length > 0 ||
                queuedSyncRef.current;

            pendingSyncRef.current = hasNewerEdits;
            isOfflineRef.current = false;
            setIsOffline(false);
            persistCurrentState();
            return true;
        } catch {
            pendingSyncRef.current = true;
            isOfflineRef.current = true;
            setIsOffline(true);
            persistCurrentState();
            return false;
        } finally {
            inFlightRef.current = false;
            setIsSyncing(false);

            if (queuedSyncRef.current && !isOfflineRef.current) {
                queuedSyncRef.current = false;
                void syncNoteToBackend();
            } else {
                queuedSyncRef.current = false;
            }
        }
    }, [createOperatorNoteMutation, updateOperatorNoteMutation, persistCurrentState]);

    const applyServerNodes = useCallback(
        (edges: any[] | undefined) => {
            const stored = loadScratchpadState();
            if (stored.pendingSync && !pendingSyncRef.current) {
                contentRef.current = stored.content;
                setContent(stored.content);
                pendingSyncRef.current = true;
                for (const sId of stored.pendingShellIds) {
                    if (!pendingShellIdsRef.current.includes(sId)) {
                        pendingShellIdsRef.current.push(sId);
                    }
                }
            }

            const serverNote = selectOperatorNoteForUser(edges || [], currentUserId);
            if (serverNote) {
                if (serverNote.id !== undefined && serverNote.id !== null) {
                    noteIdRef.current = String(serverNote.id);
                }
                const serverShells = extractShellIdsFromNote(serverNote);
                knownShellIdsRef.current = new Set(serverShells);
            }

            initialLoadDoneRef.current = true;

            if (pendingSyncRef.current) {
                if (!inFlightRef.current) {
                    void syncNoteToBackend();
                }
                return;
            }

            isOfflineRef.current = false;
            setIsOffline(false);

            if (serverNote) {
                const nextContent =
                    typeof serverNote.content === "string" ? serverNote.content : "";
                contentRef.current = nextContent;
                setContent(nextContent);
            }
            persistCurrentState();
        },
        [currentUserId, persistCurrentState, syncNoteToBackend]
    );

    const { refetch } = useQuery(GET_OPERATOR_NOTES, {
        fetchPolicy: "network-only",
        notifyOnNetworkStatusChange: true,
        onCompleted: (data) => {
            applyServerNodes(data?.operatorNotes?.edges);
        },
        onError: () => {
            initialLoadDoneRef.current = true;
            isOfflineRef.current = true;
            setIsOffline(true);
            const stored = loadScratchpadState();
            if (stored.content && !contentRef.current) {
                contentRef.current = stored.content;
                setContent(stored.content);
            }
        },
    });

    const handleRetrySync = useCallback(async () => {
        try {
            const res = await refetch();
            if (res?.data?.operatorNotes?.edges !== undefined) {
                applyServerNodes(res.data.operatorNotes.edges);
                return;
            }
        } catch {
            // If refetch is unavailable or fails, fall back to direct sync if pending
        }
        if (pendingSyncRef.current) {
            await syncNoteToBackend();
        }
    }, [refetch, applyServerNodes, syncNoteToBackend]);

    useEffect(() => {
        const handleOnline = () => {
            if (isOfflineRef.current || pendingSyncRef.current) {
                void handleRetrySync();
            }
        };
        window.addEventListener("online", handleOnline);
        return () => window.removeEventListener("online", handleOnline);
    }, [handleRetrySync]);

    useEffect(() => {
        const handleSyncEvent = (e: Event) => {
            const detail = (e as CustomEvent<StoredScratchpadState>).detail;
            if (!detail) return;
            if (detail.content !== contentRef.current) {
                contentRef.current = detail.content;
                setContent(detail.content);
            }
            if (detail.noteId) {
                noteIdRef.current = detail.noteId;
            }
            for (const sId of detail.shellIds || []) {
                knownShellIdsRef.current.add(String(sId));
            }
        };
        window.addEventListener(SCRATCHPAD_SYNC_EVENT, handleSyncEvent);
        return () => window.removeEventListener(SCRATCHPAD_SYNC_EVENT, handleSyncEvent);
    }, []);

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => textareaRef.current?.focus(), 20);
        }
    }, [isOpen]);

    useEffect(() => {
        return () => {
            if (debounceTimerRef.current) {
                clearTimeout(debounceTimerRef.current);
                debounceTimerRef.current = null;
            }
        };
    }, []);

    const recordShellModification = useCallback((activeShellId?: string) => {
        if (!activeShellId) return;
        const normalized = String(activeShellId).trim();
        if (!normalized) return;
        if (knownShellIdsRef.current.has(normalized)) {
            return;
        }
        if (!pendingShellIdsRef.current.includes(normalized)) {
            pendingShellIdsRef.current = [...pendingShellIdsRef.current, normalized];
        }
    }, []);

    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const nextValue = e.target.value;
        contentRef.current = nextValue;
        setContent(nextValue);

        pendingSyncRef.current = true;
        recordShellModification(shellId);
        persistCurrentState();

        if (debounceTimerRef.current) {
            clearTimeout(debounceTimerRef.current);
            debounceTimerRef.current = null;
        }

        if (isOfflineRef.current || !initialLoadDoneRef.current) {
            return;
        }

        debounceTimerRef.current = setTimeout(() => {
            debounceTimerRef.current = null;
            void syncNoteToBackend();
        }, SAVE_DEBOUNCE_MS);
    };

    const handleBlur = () => {
        if (debounceTimerRef.current) {
            clearTimeout(debounceTimerRef.current);
            debounceTimerRef.current = null;
        }
        if (pendingSyncRef.current && !isOfflineRef.current && initialLoadDoneRef.current) {
            void syncNoteToBackend();
        }
    };

    if (!isOpen) {
        return null;
    }

    return (
        <div
            data-testid="shell-scratchpad"
            data-tour="shell-scratchpad"
            className="flex flex-col border-t border-gray-800 bg-gray-900 overflow-hidden relative"
        >
            <div
                data-testid="scratchpad-resize-handle"
                role="separator"
                aria-orientation="horizontal"
                aria-label="Resize scratchpad"
                tabIndex={0}
                onMouseDown={handleResizeMouseDown}
                onKeyDown={handleResizeKeyDown}
                title="Drag to resize scratchpad height"
                className="absolute inset-x-0 top-0 h-1 bg-transparent hover:bg-purple-500/50 cursor-ns-resize z-20 transition-colors"
            />

            <div className="flex items-center justify-between gap-2 px-3 py-1 bg-gray-900 border-b border-gray-800 text-xs text-gray-400 select-none">
                <div className="flex items-center gap-1.5">
                    <span
                        className="inline-flex items-center justify-center p-1 text-purple-400"
                        title="Scratchpad"
                    >
                        <NotebookPen className="w-4 h-4" />
                    </span>
                    <span className="font-medium text-gray-300">Scratchpad</span>

                    {isOffline && (
                        <button
                            type="button"
                            data-testid="scratchpad-offline-badge"
                            onClick={() => void handleRetrySync()}
                            title="Offline mode — notes are saved locally and will sync when available (click to retry)"
                            className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-300 hover:bg-amber-500/25 text-[11px] font-medium transition-colors"
                        >
                            <WifiOff className="w-3 h-3 text-amber-400" />
                            <span>Offline</span>
                        </button>
                    )}

                    {!isOffline && isSyncing && (
                        <span
                            data-testid="scratchpad-syncing-indicator"
                            className="inline-flex items-center gap-1 text-[11px] text-gray-400"
                        >
                            <RefreshCw className="w-3 h-3 animate-spin" />
                        </span>
                    )}
                </div>

                <div className="flex items-center gap-1.5">
                    <button
                        type="button"
                        onClick={onToggle}
                        aria-label="Close scratchpad"
                        title="Close scratchpad"
                        className="inline-flex items-center justify-center p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
            </div>

            <textarea
                ref={textareaRef}
                data-testid="scratchpad-textarea"
                aria-label="Scratchpad notes"
                value={content}
                onChange={handleChange}
                onBlur={handleBlur}
                placeholder="Write notes here..."
                style={{
                    height: `${editorHeight}px`,
                    ...(fontSize !== undefined
                        ? {
                            fontSize: `${fontSize}px`,
                            lineHeight: `${Math.max(10, Math.round(fontSize * 1.5))}px`,
                        }
                        : {}),
                }}
                className="w-full bg-[#0f1118] text-gray-100 placeholder-gray-500 font-mono text-xs px-3 py-2 focus:outline-none resize-none overflow-y-auto shell-scrollbar"
            />
        </div>
    );
};

export default ShellScratchpad;
