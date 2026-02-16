import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { HeadlessWasmAdapter } from "../../lib/headless-adapter";

// --- Types ---
interface HistoryState {
    entries: string[];
    index: number | null;
    savedCurrentLine: string;
}

interface SearchState {
    active: boolean;
    query: string;
    result: string | null;
}

interface CompletionsState {
    list: string[];
    selected: number;
    active: boolean;
    left: number;
    top: number;
}

// --- Component ---
const ShellV2 = () => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<HeadlessWasmAdapter | null>(null);
    const fitAddon = useRef<FitAddon | null>(null);

    // --- State (Refs) ---
    const buffer = useRef<string>("");
    const cursor = useRef<number>(0);
    const history = useRef<HistoryState>({ entries: [], index: null, savedCurrentLine: "" });
    const search = useRef<SearchState>({ active: false, query: "", result: null });
    const prompt = useRef<string>(">>> ");
    const isReady = useRef<boolean>(false);

    // Completions Logic State
    const completionsRef = useRef<CompletionsState>({
        list: [], selected: 0, active: false, left: 0, top: 0
    });

    // --- UI State ---
    const [completionsUI, setCompletionsUI] = useState<CompletionsState>({
        list: [], selected: 0, active: false, left: 0, top: 0
    });

    // --- Effect: Setup ---
    useEffect(() => {
        if (!termRef.current) return;

        // Helpers inside effect to access refs and avoid dependency issues
        const updateCompletions = (newState: Partial<CompletionsState>) => {
            completionsRef.current = { ...completionsRef.current, ...newState };
            setCompletionsUI({ ...completionsRef.current });
        };

        const refreshLine = () => {
            if (!termInstance.current) return;
            const term = termInstance.current;

            if (search.current.active) {
                term.write(`\x1b[2K\r(reverse-i-search)\`${search.current.query}': ${search.current.result || ''}`);
                return;
            }

            term.write("\x1b[2K\r"); // Clear line and return
            term.write(prompt.current + buffer.current);

            const promptLen = prompt.current.length;
            const totalLen = promptLen + buffer.current.length;
            const targetCol = promptLen + cursor.current;

            const dist = totalLen - targetCol;
            if (dist > 0) {
                term.write(`\x1b[${dist}D`);
            }
        };

        const insertText = (text: string) => {
            const b = buffer.current;
            const c = cursor.current;
            buffer.current = b.slice(0, c) + text + b.slice(c);
            cursor.current += text.length;
            refreshLine();
        };

        const deleteCharBack = () => {
            if (cursor.current > 0) {
                const b = buffer.current;
                const c = cursor.current;
                buffer.current = b.slice(0, c - 1) + b.slice(c);
                cursor.current--;
                refreshLine();
            }
        };

        const deleteWordBack = () => {
            if (cursor.current === 0) return;
            const b = buffer.current;
            let c = cursor.current;
            while (c > 0 && b[c - 1] === ' ') c--;
            while (c > 0 && b[c - 1] !== ' ') c--;
            buffer.current = b.slice(0, c) + b.slice(cursor.current);
            cursor.current = c;
            refreshLine();
        };

        const clearLine = () => {
            buffer.current = "";
            cursor.current = 0;
            refreshLine();
        };

        const moveCursor = (delta: number) => {
            const newCursor = cursor.current + delta;
            if (newCursor >= 0 && newCursor <= buffer.current.length) {
                cursor.current = newCursor;
                refreshLine();
            }
        };

        const historyNavigate = (direction: -1 | 1) => {
            const h = history.current;
            if (h.entries.length === 0) return;
            if (h.index === null) {
                if (direction === -1) {
                    h.savedCurrentLine = buffer.current;
                    h.index = h.entries.length - 1;
                }
            } else {
                const newIndex = h.index + direction;
                if (newIndex >= 0 && newIndex < h.entries.length) {
                    h.index = newIndex;
                } else if (newIndex === h.entries.length) {
                    h.index = null;
                }
            }
            if (h.index !== null) {
                buffer.current = h.entries[h.index];
            } else {
                buffer.current = h.savedCurrentLine;
            }
            cursor.current = buffer.current.length;
            refreshLine();
        };

        const performSearch = (char?: string) => {
            if (char) search.current.query += char;
            const query = search.current.query;
            let found = null;
            for (let i = history.current.entries.length - 1; i >= 0; i--) {
                if (history.current.entries[i].includes(query)) {
                    found = history.current.entries[i];
                    break;
                }
            }
            search.current.result = found;
            refreshLine();
        };

        // We expose this for the render function if needed, but it's simpler to keep logic here
        // But the Dropdown onClick needs access to acceptCompletion.
        // So we need to store it in a ref or something?
        // Or simply define it outside?
        // If we define inside, we can't use it in JSX outside.
        // We can use a ref to hold the function.
        // See below.

        termInstance.current = new Terminal({
            cursorBlink: true,
            theme: { background: "#1e1e1e", foreground: "#d4d4d4" },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 14,
            allowProposedApi: true
        });

        fitAddon.current = new FitAddon();
        termInstance.current.loadAddon(fitAddon.current);
        termInstance.current.open(termRef.current);
        fitAddon.current.fit();

        termInstance.current.write("Initializing Headless REPL...\r\n");

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws`;

        adapter.current = new HeadlessWasmAdapter(url, (content) => {
            if (!termInstance.current) return;
            const formatted = content.replace(/\n/g, "\r\n");

            termInstance.current.write(formatted);
            if (!formatted.endsWith("\n") && !formatted.endsWith("\r")) {
                termInstance.current.write("\r\n");
            }

            termInstance.current.write(prompt.current + buffer.current);

            const totalLen = prompt.current.length + buffer.current.length;
            const targetCol = prompt.current.length + cursor.current;
            const dist = totalLen - targetCol;
            if (dist > 0) {
                termInstance.current.write(`\x1b[${dist}D`);
            }

        }, () => {
            termInstance.current?.write("Connected.\r\n" + prompt.current);
            isReady.current = true;
        });

        adapter.current.init();

        const handleKey = (e: { key: string, domEvent: KeyboardEvent }) => {
            if (!termInstance.current || !isReady.current) return;

            const ev = e.domEvent;
            const keyName = ev.key;

            if (completionsRef.current.active) {
                if (keyName === "Escape" || (ev.ctrlKey && (keyName === "c" || keyName === "C"))) {
                    updateCompletions({ active: false });
                    return;
                }
                if (keyName === "ArrowDown") {
                    const next = (completionsRef.current.selected + 1) % completionsRef.current.list.length;
                    updateCompletions({ selected: next });
                    ev.preventDefault();
                    return;
                }
                if (keyName === "ArrowUp") {
                    const prev = (completionsRef.current.selected - 1 + completionsRef.current.list.length) % completionsRef.current.list.length;
                    updateCompletions({ selected: prev });
                    ev.preventDefault();
                    return;
                }
                if (keyName === "Enter" || keyName === "Tab") {
                    ev.preventDefault();
                    acceptCompletionRef.current(completionsRef.current.list[completionsRef.current.selected]);
                    return;
                }
                updateCompletions({ active: false });
            }

            if (search.current.active) {
                if (ev.ctrlKey && (keyName === "c" || keyName === "C")) {
                    search.current.active = false;
                    search.current.query = "";
                    search.current.result = null;
                    refreshLine();
                    return;
                }
                if (keyName === "Enter") {
                    if (search.current.result !== null) {
                        buffer.current = search.current.result;
                        cursor.current = buffer.current.length;
                    }
                    search.current.active = false;
                    refreshLine();
                    return;
                }
                if (keyName === "Backspace") {
                    search.current.query = search.current.query.slice(0, -1);
                    performSearch();
                    return;
                }
                if (e.key.length === 1 && !ev.ctrlKey && !ev.altKey && !ev.metaKey) {
                    performSearch(e.key);
                    return;
                }
                return;
            }

            if (ev.ctrlKey) {
                const k = keyName.toLowerCase();
                if (["c", "l", "u", "w", "a", "e", "r"].includes(k)) {
                     ev.preventDefault();
                }

                switch (k) {
                    case "c":
                        adapter.current?.clear();
                        termInstance.current.write("^C\r\n");
                        buffer.current = "";
                        cursor.current = 0;
                        prompt.current = ">>> ";
                        termInstance.current.write(prompt.current);
                        break;
                    case "l":
                        termInstance.current.clear();
                        termInstance.current.write(prompt.current + buffer.current);
                        break;
                    case "u":
                        clearLine();
                        break;
                    case "w":
                        deleteWordBack();
                        break;
                    case "a":
                        cursor.current = 0;
                        refreshLine();
                        break;
                    case "e":
                        cursor.current = buffer.current.length;
                        refreshLine();
                        break;
                    case "r":
                        search.current.active = true;
                        search.current.query = "";
                        search.current.result = null;
                        refreshLine();
                        break;
                }
                return;
            }

            switch (keyName) {
                case "Enter": {
                    const line = buffer.current;
                    termInstance.current.write("\r\n");

                    const res = adapter.current?.input(line);

                    if (res?.status === "complete") {
                        if (line.trim().length > 0) {
                            history.current.entries.push(line);
                            history.current.index = null;
                        }
                        buffer.current = "";
                        cursor.current = 0;
                    } else if (res?.status === "incomplete") {
                        prompt.current = res.prompt || ".. ";
                        buffer.current = "";
                        cursor.current = 0;
                        termInstance.current.write(prompt.current);
                    } else if (res?.status === "error") {
                        termInstance.current.write(`Error: ${res.message}\r\n`);
                        prompt.current = ">>> ";
                        buffer.current = "";
                        cursor.current = 0;
                        termInstance.current.write(prompt.current);
                    }
                    break;
                }
                case "Backspace":
                    deleteCharBack();
                    break;
                case "ArrowLeft":
                    moveCursor(-1);
                    break;
                case "ArrowRight":
                    moveCursor(1);
                    break;
                case "ArrowUp":
                    historyNavigate(-1);
                    break;
                case "ArrowDown":
                    historyNavigate(1);
                    break;
                case "Tab":
                    ev.preventDefault();
                    if (buffer.current.trim().length === 0) {
                        insertText("    ");
                    } else {
                        const suggestions = adapter.current?.complete(buffer.current, cursor.current);
                        if (suggestions && suggestions.length > 0) {
                            const term = termInstance.current;
                            // @ts-ignore
                            const cursorX = term.buffer.active.cursorX;
                            // @ts-ignore
                            const cursorY = term.buffer.active.cursorY;

                            updateCompletions({
                                list: suggestions,
                                selected: 0,
                                active: true,
                                left: cursorX * 9 + 20,
                                top: (cursorY + 1) * 17 + 20
                            });
                        }
                    }
                    break;
                default:
                    if (e.key.length === 1) {
                        insertText(e.key);
                    }
                    break;
            }
        };

        const onKey = termInstance.current.onKey(handleKey);

        const handleResize = () => {
            fitAddon.current?.fit();
        };
        window.addEventListener("resize", handleResize);

        // Store acceptCompletion in ref so outside JSX can call it
        acceptCompletionRef.current = (text: string) => {
             const left = buffer.current.slice(0, cursor.current);
             let overlap = 0;
             for (let i = 1; i <= text.length; i++) {
                 const sub = text.slice(0, i);
                 if (left.endsWith(sub)) {
                     overlap = i;
                 }
             }
             const suffix = text.slice(overlap);
             insertText(suffix);
             updateCompletions({ active: false });
        };

        return () => {
            onKey.dispose();
            adapter.current?.close();
            termInstance.current?.dispose();
            window.removeEventListener("resize", handleResize);
        };
    }, []);

    // Ref to hold acceptCompletion function
    const acceptCompletionRef = useRef<(text: string) => void>(() => {});

    return (
        <div style={{ padding: "20px", height: "calc(100vh - 100px)", position: "relative" }}>
            <h1 className="text-xl font-bold mb-4">Shell V2 (Headless REPL)</h1>
            <div ref={termRef} style={{ height: "100%", width: "100%" }} />

            {completionsUI.active && (
                <div style={{
                    position: "absolute",
                    left: completionsUI.left,
                    top: completionsUI.top,
                    backgroundColor: "#252526",
                    border: "1px solid #454545",
                    zIndex: 1000,
                    boxShadow: "0 4px 6px rgba(0,0,0,0.3)"
                }}>
                    {completionsUI.list.map((item, idx) => (
                        <div key={item} style={{
                            padding: "4px 8px",
                            cursor: "pointer",
                            backgroundColor: idx === completionsUI.selected ? "#04395e" : "transparent",
                            color: "#cccccc",
                            fontFamily: "monospace"
                        }}
                        onMouseDown={(e) => {
                           e.preventDefault();
                        }}
                        onClick={() => {
                            acceptCompletionRef.current(item);
                        }}
                        >
                            {item}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default ShellV2;
