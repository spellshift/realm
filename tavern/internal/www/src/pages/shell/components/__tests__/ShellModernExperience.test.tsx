import React from "react";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import "@testing-library/jest-dom";

import { AuthorizationContext } from "../../../../context/AuthorizationContext";
import { resetSharedDrawerHeightForTests } from "../EldritchCodeEditor";
import InterpreterStatusBanner from "../InterpreterStatusBanner";
import MultiLineComposer, {
    DEFAULT_RC_ALIASES_SCRIPT,
    DEFAULT_SHELL_FONT_SIZE,
    MAX_SHELL_FONT_SIZE,
    MIN_SHELL_FONT_SIZE,
    clampShellFontSize,
    loadRcEnabled,
    loadRcScript,
    loadShellFontSize,
} from "../MultiLineComposer";
import {
    CREATE_CREDENTIAL_MUTATION,
    CREATE_NETWORK_HOST_MUTATION,
    CREATE_OPERATOR_NOTE_MUTATION,
    CREATE_TOME_MUTATION,
    GET_OPERATOR_NOTES,
    GET_SHELL_TOMES,
    UPDATE_NETWORK_HOST_MUTATION,
    UPDATE_OPERATOR_NOTE_MUTATION,
} from "../../graphql";
import { parseComposerScriptForTome } from "../../utils/tomeComposerUtils";
import SessionSettings from "../SessionSettings";
import ShellHistoryDrawer from "../ShellHistoryDrawer";
import ShellPivotsDrawer from "../ShellPivotsDrawer";
import HostsTreeDrawer from "../../../../components/hosts-tree/HostsTreeDrawer";
import CredentialModal from "../graph/CredentialModal";
import HostModal from "../graph/HostModal";
import ShellScratchpad, {
    SCRATCHPAD_STORAGE_KEY,
    loadScratchpadState,
} from "../ShellScratchpad";
import ShellTerminal from "../ShellTerminal";
import ShellWalkthrough from "../ShellWalkthrough";
import SshConnectionModal from "../SshConnectionModal";
import { formatPivotTabLabel } from "../../index";
import {
    DEPRECATED_STDLIB_FUNCTIONS,
    ELDRITCH_BOOLEANS,
    ELDRITCH_BUILTINS,
    ELDRITCH_KEYWORDS,
    ELDRITCH_MODULES,
    analyzeEldritchCode,
    clearUserDefinedSymbols,
    completeEldritchCode,
    getEldritchSignatureHelp,
    getUserDefinedFunctions,
    getUserDefinedVariables,
    registerUserSymbolsFromCode,
    stdlibRegistry,
} from "../../utils/eldritchLsp";
import {
    buildUnifiedGraphData,
    extractPivotTargetHost,
    formatFileSize,
    formatPortListInput,
    formatSubnetCidr,
    getIpv4SubnetPrefix,
    hasPivotActivity,
    isBeaconOnline,
    parsePortList,
    togglePortInList,
} from "../../utils/networkGraphUtils";
import {
    BrowserWasmAdapter,
    WEBSOCKET_CONNECT_TIMEOUT_MS,
    getExponentialBackoffDelay,
    normalizeMultilineScript,
} from "../../../../lib/browser-adapter";
import {
    KnowledgeCredentialItem,
    KnowledgeHostItem,
    ShellPivotItem,
    ShellTaskItem,
    ShellTomeItem,
} from "../../hooks/useShellData";
import {
    isTerminalElementVisible,
    transformClearAnsiForScrollback,
} from "../../hooks/shellUtils";

describe("ShellWalkthrough & InterpreterStatusBanner", () => {
    const baseProps = {
        closedAt: null as string | null,
        connectionStatus: "connected" as const,
        connectionMessage: "",
        stuckTask: null as { task: ShellTaskItem; elapsedSeconds: number } | null,
        queuedTasks: [] as ShellTaskItem[],
        runningTasks: [] as ShellTaskItem[],
        timeUntilCallback: "in 8 minutes",
        onSpawnFreshShell: vi.fn(),
        isSpawningShell: false,
    };

    it("renders 1-time interactive walkthrough popover with component steps, opens composer on step 2, and dismisses cleanly", () => {
        const onDismiss = vi.fn();
        const onOpenGuideDrawer = vi.fn();
        const onOpenComposer = vi.fn();
        const onCloseComposer = vi.fn();
        const onOpenScratchpad = vi.fn();
        const onCloseScratchpad = vi.fn();
        const onOpenSessionSettings = vi.fn();
        const onPortalStepChange = vi.fn();
        const onQueueStepChange = vi.fn();

        render(
            <ShellWalkthrough
                isOpen={true}
                onDismiss={onDismiss}
                onOpenGuideDrawer={onOpenGuideDrawer}
                onOpenComposer={onOpenComposer}
                onCloseComposer={onCloseComposer}
                onOpenScratchpad={onOpenScratchpad}
                onCloseScratchpad={onCloseScratchpad}
                onOpenSessionSettings={onOpenSessionSettings}
                onPortalStepChange={onPortalStepChange}
                onQueueStepChange={onQueueStepChange}
            />
        );

        expect(screen.getByTestId("shell-nux-banner")).toBeInTheDocument();
        expect(screen.getByText("Welcome to the Eldritch REPL")).toBeInTheDocument();
        expect(screen.getByText(/You can use the ! macro to quickly run shell commands, such as !ls/i)).toBeInTheDocument();
        expect(onOpenComposer).not.toHaveBeenCalled();

        // Step 2: RC Script (second NUX in the flow, pointing to cogwheel)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("RC Script")).toBeInTheDocument();
        expect(onOpenSessionSettings).not.toHaveBeenCalled();

        // Step 3: Set Default Aliases (opens SessionSettings drawer and highlights Set Default Aliases button)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Set Default Aliases")).toBeInTheDocument();
        expect(onOpenSessionSettings).toHaveBeenCalledTimes(1);

        // Step 4: Eldritch Composer (opens composer on bottom-left bar)
        onCloseComposer.mockClear();
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Eldritch Composer")).toBeInTheDocument();
        expect(onOpenComposer).toHaveBeenCalledTimes(1);
        expect(onCloseComposer).not.toHaveBeenCalled();

        // Step 5: Scratchpad (closes composer and opens scratchpad right after composer)
        onCloseScratchpad.mockClear();
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Scratchpad")).toBeInTheDocument();
        expect(
            screen.getByText(
                "A place for you to store notes, which persists across sessions for easy access."
            )
        ).toBeInTheDocument();
        expect(onCloseComposer).toHaveBeenCalledTimes(1);
        expect(onOpenScratchpad).toHaveBeenCalledTimes(1);
        expect(onCloseScratchpad).not.toHaveBeenCalled();

        // Step 6: Agent Callback (closes scratchpad and highlights Next on bottom-right bar)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Agent Callback")).toBeInTheDocument();
        expect(onCloseScratchpad).toHaveBeenCalledTimes(1);

        // Step 7: Async Mode (positioned above bottom status bar element without overlapping)
        const nonInteractiveEl = document.createElement("span");
        nonInteractiveEl.setAttribute("data-tour", "shell-non-interactive");
        nonInteractiveEl.getBoundingClientRect = () =>
        ({
            top: 760,
            left: 24,
            width: 110,
            height: 20,
            bottom: 780,
            right: 134,
            x: 24,
            y: 760,
            toJSON: () => { },
        } as DOMRect);
        document.body.appendChild(nonInteractiveEl);

        fireEvent.click(screen.getByText("Next"));
        const dialog = screen.getByRole("dialog", { name: "Async Mode" });
        expect(dialog).toBeInTheDocument();
        expect(dialog.style.bottom).toBe(`${window.innerHeight - 760 + 14}px`);
        expect(dialog.style.top).toBe("");
        document.body.removeChild(nonInteractiveEl);

        // Step 8: Access
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Access")).toBeInTheDocument();
        expect(
            screen.getByText(
                /Navigate hosts, shells, pivots, and PTYs, add target hosts or credentials, and jump straight into SSH pivots from one unified Access drawer\./i
            )
        ).toBeInTheDocument();

        // Step 9: Going Interactive (Portal in status bar)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Going Interactive")).toBeInTheDocument();
        expect(onPortalStepChange).toHaveBeenLastCalledWith(false);

        // Step 10: History
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByRole("dialog", { name: "History" })).toBeInTheDocument();
        expect(
            screen.getByText(/across all of your shell sessions, not just the active one/i)
        ).toBeInTheDocument();
        expect(screen.queryByText("Tip:")).not.toBeInTheDocument();

        // Step 11: Eldritch Documentation (opens ... actions menu)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Eldritch Documentation")).toBeInTheDocument();
        expect(onPortalStepChange).toHaveBeenLastCalledWith(true);

        // Step 12: Feedback & Bug Reports (in ... actions menu)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Feedback & Bug Reports")).toBeInTheDocument();
        expect(onPortalStepChange).toHaveBeenLastCalledWith(true);
        expect(screen.getByText(/Thank you for helping make Realm better!/i)).toBeInTheDocument();

        fireEvent.click(screen.getByLabelText("Dismiss welcome guide"));
        expect(onDismiss).toHaveBeenCalled();
    });

    it("cleanly closes SessionSettings and opens MultiLineComposer without stacking when moving from settings to composer during the NUX", () => {
        const sampleAliasesTome: ShellTomeItem = {
            id: "tome-aliases",
            name: "Shell Aliases",
            description: "Default aliases for shell sessions",
            eldritch: 'def ls():\n    print("alias ls")\n',
            tactic: "SHELL_RC",
        };

        const StatefulNuxBottomPanelsHarness: React.FC = () => {
            const [activeBottomPanel, setActiveBottomPanel] = React.useState<
                "composer" | "settings" | "scratchpad" | null
            >(null);
            const [composerCode, setComposerCode] = React.useState("");
            const [rcCode, setRcCode] = React.useState("");
            const [rcEnabled, setRcEnabled] = React.useState(false);

            const composerOpen = activeBottomPanel === "composer";
            const sessionSettingsOpen = activeBottomPanel === "settings";

            return (
                <div>
                    <ShellWalkthrough
                        isOpen={true}
                        onDismiss={vi.fn()}
                        onOpenComposer={() => setActiveBottomPanel("composer")}
                        onCloseComposer={() =>
                            setActiveBottomPanel((prev) => (prev === "composer" ? null : prev))
                        }
                        onOpenScratchpad={() => setActiveBottomPanel("scratchpad")}
                        onCloseScratchpad={() =>
                            setActiveBottomPanel((prev) => (prev === "scratchpad" ? null : prev))
                        }
                        onOpenSessionSettings={() => setActiveBottomPanel("settings")}
                        onCloseSessionSettings={() =>
                            setActiveBottomPanel((prev) => (prev === "settings" ? null : prev))
                        }
                    />
                    <MultiLineComposer
                        isOpen={composerOpen}
                        onToggle={() =>
                            setActiveBottomPanel((prev) =>
                                prev === "composer" ? null : "composer"
                            )
                        }
                        code={composerCode}
                        onChange={setComposerCode}
                        onRunCode={vi.fn()}
                        tomes={[sampleAliasesTome]}
                    />
                    <SessionSettings
                        isOpen={sessionSettingsOpen}
                        onToggle={() =>
                            setActiveBottomPanel((prev) =>
                                prev === "settings" ? null : "settings"
                            )
                        }
                        onClose={() =>
                            setActiveBottomPanel((prev) => (prev === "settings" ? null : prev))
                        }
                        tomes={[sampleAliasesTome]}
                        rcCode={rcCode}
                        onRcCodeChange={setRcCode}
                        rcEnabled={rcEnabled}
                        onRcEnabledChange={setRcEnabled}
                    />
                    <button
                        type="button"
                        data-tour="shell-session-settings-toggle"
                        onClick={() =>
                            setActiveBottomPanel((prev) =>
                                prev === "settings" ? null : "settings"
                            )
                        }
                    >
                        Settings Toggle
                    </button>
                    <button
                        type="button"
                        data-tour="shell-composer-toggle"
                        onClick={() =>
                            setActiveBottomPanel((prev) =>
                                prev === "composer" ? null : "composer"
                            )
                        }
                    >
                        Composer Toggle
                    </button>
                </div>
            );
        };

        render(<StatefulNuxBottomPanelsHarness />);

        // Step 1 -> Step 2 (RC Script)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByRole("dialog", { name: "RC Script" })).toBeInTheDocument();

        // Clicking the Settings toggle button advances to Step 3 (Set Default Aliases) and opens SessionSettings
        fireEvent.click(screen.getByText("Settings Toggle"));
        expect(screen.getByRole("dialog", { name: "Set Default Aliases" })).toBeInTheDocument();
        expect(screen.getByTestId("session-settings-drawer")).toBeInTheDocument();
        expect(screen.queryByTestId("multiline-composer")).not.toBeInTheDocument();

        // Clicking "Set Default Aliases" inside SessionSettings closes SessionSettings and opens MultiLineComposer on Step 4
        const setupAliasesBtn = document.querySelector<HTMLElement>(
            '[data-tour="shell-setup-aliases"]'
        );
        expect(setupAliasesBtn).not.toBeNull();
        fireEvent.click(setupAliasesBtn!);

        expect(screen.getByRole("dialog", { name: "Eldritch Composer" })).toBeInTheDocument();
        expect(screen.getByTestId("multiline-composer")).toBeInTheDocument();
        expect(screen.queryByTestId("session-settings-drawer")).not.toBeInTheDocument();

        // Going Back to Step 3 re-opens SessionSettings and cleanly closes MultiLineComposer
        fireEvent.click(screen.getByText("Back"));
        expect(screen.getByRole("dialog", { name: "Set Default Aliases" })).toBeInTheDocument();
        expect(screen.getByTestId("session-settings-drawer")).toBeInTheDocument();
        expect(screen.queryByTestId("multiline-composer")).not.toBeInTheDocument();

        // Clicking Next from Step 3 to Step 4 also cleanly closes SessionSettings and opens MultiLineComposer
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByRole("dialog", { name: "Eldritch Composer" })).toBeInTheDocument();
        expect(screen.getByTestId("multiline-composer")).toBeInTheDocument();
        expect(screen.queryByTestId("session-settings-drawer")).not.toBeInTheDocument();
    });

    it("avoids redundant executing spinners in banner when tasks are queued or running, and supports manual reconnect", () => {
        const queuedTask: ShellTaskItem = {
            id: "t1",
            createdAt: new Date().toISOString(),
            input: 'print("queued cmd")',
        };
        const onReconnect = vi.fn();

        const { rerender } = render(
            <InterpreterStatusBanner {...baseProps} queuedTasks={[queuedTask]} />
        );

        expect(screen.queryByTestId("queued-tasks-strip")).not.toBeInTheDocument();
        expect(screen.queryByTestId("stuck-interpreter-banner")).not.toBeInTheDocument();

        // While initial WebSocket connection is in flight (isConnecting=true), do NOT show disconnection banner
        rerender(
            <InterpreterStatusBanner
                {...baseProps}
                connectionStatus="disconnected"
                isConnecting={true}
                onReconnect={onReconnect}
            />
        );
        expect(screen.queryByTestId("shell-ws-banner")).not.toBeInTheDocument();

        // Once initial connection finishes and socket is disconnected (isConnecting=false), show banner
        rerender(
            <InterpreterStatusBanner
                {...baseProps}
                connectionStatus="disconnected"
                isConnecting={false}
                onReconnect={onReconnect}
            />
        );
        const reconnectBtn = screen.getByText(/Reconnect Now/i);
        expect(reconnectBtn).toBeInTheDocument();
        fireEvent.click(reconnectBtn);
        expect(onReconnect).toHaveBeenCalled();

        // ShellTerminal renders a large icon-only spinner in place of the terminal window while isConnecting=true
        const termRef = React.createRef<HTMLDivElement>();
        const { rerender: rerenderTerm } = render(
            <ShellTerminal
                termRef={termRef}
                isConnecting={true}
                completions={[]}
                showCompletions={false}
                completionPos={{ x: 0, y: 0 }}
                completionIndex={0}
                onMouseMove={vi.fn()}
                tooltipState={{ visible: false, x: 0, y: 0, signature: "", description: "" }}
                handleCompletionSelect={vi.fn()}
            />
        );
        const loadingSpinner = screen.getByTestId("shell-terminal-loading");
        expect(loadingSpinner).toBeInTheDocument();
        expect(loadingSpinner.textContent).toBe("");

        // When connected and commands are queued/executing (isPromptBusy=true), renders a small spinner overlay with '>' at the prompt
        rerenderTerm(
            <ShellTerminal
                termRef={termRef}
                isConnecting={false}
                isPromptBusy={true}
                isMissedCallback={false}
                promptPos={{ top: 24, left: 6, width: 24, height: 18, visible: true }}
                completions={[]}
                showCompletions={false}
                completionPos={{ x: 0, y: 0 }}
                completionIndex={0}
                onMouseMove={vi.fn()}
                tooltipState={{ visible: false, x: 0, y: 0, signature: "", description: "" }}
                handleCompletionSelect={vi.fn()}
            />
        );
        const promptSpinner = screen.getByTestId("shell-prompt-spinner");
        expect(promptSpinner).toBeInTheDocument();
        expect(promptSpinner).toHaveTextContent(">");
        expect(promptSpinner.style.top).toBe("24px");
        expect(promptSpinner.style.left).toBe("6px");
        expect(screen.getByTestId("shell-prompt-spinner-icon")).toHaveClass("text-amber-400");

        // Turns red when beacon misses a callback
        rerenderTerm(
            <ShellTerminal
                termRef={termRef}
                isConnecting={false}
                isPromptBusy={true}
                isMissedCallback={true}
                promptPos={{ top: 24, left: 6, width: 24, height: 18, visible: true }}
                completions={[]}
                showCompletions={false}
                completionPos={{ x: 0, y: 0 }}
                completionIndex={0}
                onMouseMove={vi.fn()}
                tooltipState={{ visible: false, x: 0, y: 0, signature: "", description: "" }}
                handleCompletionSelect={vi.fn()}
            />
        );
        expect(screen.getByTestId("shell-prompt-spinner-icon")).toHaveClass("text-red-400");
    });

    it("renders stuck interpreter warning and triggers New Interpreter", () => {
        const onSpawnFreshShell = vi.fn();
        const stuckTaskItem: ShellTaskItem = {
            id: "t-stuck",
            createdAt: new Date().toISOString(),
            claimedAt: new Date(Date.now() - 45000).toISOString(),
            input: 'sys.shell("sleep 9999")',
        };

        render(
            <InterpreterStatusBanner
                {...baseProps}
                runningTasks={[stuckTaskItem]}
                stuckTask={{ task: stuckTaskItem, elapsedSeconds: 45 }}
                onSpawnFreshShell={onSpawnFreshShell}
            />
        );

        expect(screen.getByTestId("stuck-interpreter-banner")).toBeInTheDocument();
        expect(screen.getByText('sys.shell("sleep 9999")')).toBeInTheDocument();
        expect(screen.getByText(/is taking longer than expected\. \(45s\)/i)).toBeInTheDocument();

        const newInterpreterBtn = screen.getByRole("button", { name: /New Interpreter/i });
        expect(newInterpreterBtn.querySelector("svg.lucide-terminal")).toBeInTheDocument();
        fireEvent.click(newInterpreterBtn);
        expect(onSpawnFreshShell).toHaveBeenCalled();
    });

    it("enforces a 4s WebSocket connection timeout, fast exponential backoff, and preserves CONNECTING sockets on visibilitychange", () => {
        expect(WEBSOCKET_CONNECT_TIMEOUT_MS).toBe(4_000);
        expect(getExponentialBackoffDelay(0)).toBe(250);
        expect(getExponentialBackoffDelay(1)).toBe(500);
        expect(getExponentialBackoffDelay(2)).toBe(1_000);
        expect(getExponentialBackoffDelay(3)).toBe(2_000);
        expect(getExponentialBackoffDelay(4)).toBe(4_000);
        expect(getExponentialBackoffDelay(5)).toBe(5_000);

        vi.useFakeTimers();
        const OriginalWebSocket = globalThis.WebSocket;
        const createdSockets: any[] = [];

        class MockWebSocket {
            static CONNECTING = 0;
            static OPEN = 1;
            static CLOSING = 2;
            static CLOSED = 3;
            readyState = MockWebSocket.CONNECTING;
            url: string;
            onopen: (() => void) | null = null;
            onmessage: ((ev: any) => void) | null = null;
            onclose: ((ev: any) => void) | null = null;
            onerror: ((ev: any) => void) | null = null;
            close = vi.fn(() => {
                this.readyState = MockWebSocket.CLOSED;
            });
            send = vi.fn();
            constructor(url: string) {
                this.url = url;
                createdSockets.push(this);
            }
        }

        (globalThis as any).WebSocket = MockWebSocket;
        try {
            const statuses: string[] = [];
            const adapter = new BrowserWasmAdapter(
                "ws://localhost/shell/ws?shell_id=1",
                vi.fn(),
                vi.fn(),
                (s) => statuses.push(s)
            );

            // Initial socket created; visibilitychange / online while CONNECTING must NOT abort the socket
            expect(createdSockets).toHaveLength(1);
            document.dispatchEvent(new Event("visibilitychange"));
            window.dispatchEvent(new Event("online"));
            expect(createdSockets).toHaveLength(1);

            // Does not time out at 2s
            vi.advanceTimersByTime(2_000);
            expect(createdSockets).toHaveLength(1);
            expect(statuses).toEqual([]);

            // At 4s, initial connection attempt times out and schedules retry #1 (250ms backoff)
            vi.advanceTimersByTime(2_000);
            expect(statuses).toContain("reconnecting");
            expect(createdSockets).toHaveLength(1);

            vi.advanceTimersByTime(250);
            expect(createdSockets).toHaveLength(2);

            // Second attempt also hangs for 4s -> schedules retry #2 (500ms backoff)
            vi.advanceTimersByTime(4_000);
            expect(createdSockets).toHaveLength(2);
            vi.advanceTimersByTime(499);
            expect(createdSockets).toHaveLength(2);
            vi.advanceTimersByTime(1);
            expect(createdSockets).toHaveLength(3);

            // Third attempt fails immediately via onclose -> schedules retry #3 (1s backoff)
            createdSockets[2].readyState = MockWebSocket.CLOSED;
            createdSockets[2].onclose?.({ reason: "network error" });
            vi.advanceTimersByTime(999);
            expect(createdSockets).toHaveLength(3);
            vi.advanceTimersByTime(1);
            expect(createdSockets).toHaveLength(4);

            // When connected, `input("!ls")` finalizes the expanded macro with a follow-up `input("")` call
            createdSockets[3].readyState = MockWebSocket.OPEN;
            createdSockets[3].onopen?.();
            const replInputs: string[] = [];
            (adapter as any).repl = {
                input: vi.fn((line: string) => {
                    replInputs.push(line);
                    if (line.trimStart().startsWith("!")) {
                        return JSON.stringify({ status: "incomplete", prompt: ".. " });
                    }
                    return JSON.stringify({
                        status: "complete",
                        payload:
                            'for _nonomacroclowntown in range(1):\n\t_nonomacroclowntown = sys.shell("ls")\n\tprint(_nonomacroclowntown[\'stdout\']);print(_nonomacroclowntown[\'stderr\'])\n',
                    });
                }),
                reset: vi.fn(),
            };
            const macroRes = adapter.input("!ls");
            expect(macroRes).toEqual({ status: "complete" });
            expect(replInputs).toEqual(["!ls", ""]);
            expect(createdSockets[3].send).toHaveBeenCalledTimes(1);
            expect(JSON.parse(createdSockets[3].send.mock.calls[0][0])).toEqual({
                kind: "INPUT",
                input: 'for _nonomacroclowntown in range(1):\n\t_nonomacroclowntown = sys.shell("ls")\n\tprint(_nonomacroclowntown[\'stdout\']);print(_nonomacroclowntown[\'stderr\'])\n',
            });

            adapter.close();
        } finally {
            (globalThis as any).WebSocket = OriginalWebSocket;
            vi.useRealTimers();
        }
    });
});

describe("MultiLineComposer, SessionSettings (RC Drawer & SHELL_RC Tomes) & eldritch-lsp", () => {
    const sampleTomes: ShellTomeItem[] = [
        {
            id: "tome-1",
            name: "File List",
            description: "List files and directories with metadata.",
            tactic: "DISCOVERY",
            supportModel: "FIRST_PARTY",
            paramDefs: JSON.stringify([
                { name: "path", type: "string", default: "/tmp" },
            ]),
            eldritch: 'for f in file.list(input_params["path"]):\n    print(f["file_name"])',
        },
        {
            id: "tome-2",
            name: "Process Tree",
            description: "Display running processes.",
            tactic: "DISCOVERY",
            supportModel: "FIRST_PARTY",
            paramDefs: "[]",
            eldritch: 'for p in process.list():\n    print(p["pid"], p["name"])',
        },
        {
            id: "tome-rc-1",
            name: "Shell Aliases",
            description: "Interactive Eldritch REPL helper functions (ls, whoami, ps, pwd, cat, netstat).",
            tactic: "SHELL_RC",
            supportModel: "FIRST_PARTY",
            paramDefs: "[]",
            eldritch: DEFAULT_RC_ALIASES_SCRIPT,
        },
    ];

    it("loads regular tomes in MultiLineComposer (filtering out SHELL_RC) and manages RC scripts in SessionSettings with Load RC Tome or custom chooser", () => {
        window.localStorage.clear();
        const onRunCode = vi.fn();

        expect(loadRcEnabled()).toBe(false);
        expect(loadRcScript()).toBe("");

        const onCloseSessionSettings = vi.fn();
        const StatefulComposerAndSessionSettings = () => {
            const [code, setCode] = React.useState('for p in process.list():\n    print(p["name"])');
            const [rcCode, setRcCode] = React.useState("");
            const [rcEnabled, setRcEnabled] = React.useState(false);
            return (
                <>
                    <MultiLineComposer
                        isOpen={true}
                        onToggle={vi.fn()}
                        code={code}
                        onChange={setCode}
                        onRunCode={onRunCode}
                        tomes={sampleTomes}
                    />
                    <SessionSettings
                        isOpen={true}
                        onClose={onCloseSessionSettings}
                        rcCode={rcCode}
                        onRcChange={setRcCode}
                        rcEnabled={rcEnabled}
                        onRcEnabledChange={setRcEnabled}
                        tomes={sampleTomes}
                    />
                </>
            );
        };

        render(<StatefulComposerAndSessionSettings />);

        expect(screen.getByTestId("composer-syntax-highlight")).toBeInTheDocument();
        expect(screen.queryByTestId("eldritch-lsp-status")).not.toBeInTheDocument();
        expect(screen.getByText("Eldritch")).toBeInTheDocument();
        expect(screen.getByTitle("Eldritch Composer")).toHaveClass("text-gray-400");
        expect(screen.getByTestId("composer-resize-handle")).toBeInTheDocument();
        expect(screen.queryByTestId("composer-rc-tab")).not.toBeInTheDocument();

        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(textarea.value).toContain("process.list()");

        fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });
        expect(onRunCode).toHaveBeenCalledWith('for p in process.list():\n    print(p["name"])');

        // Composer Tome picker filters out SHELL_RC tomes and replaces Script view contents
        const tomePickerBtn = screen.getByTestId("composer-tome-picker-button");
        expect(tomePickerBtn).not.toHaveTextContent("Load Tome...");
        expect(tomePickerBtn).not.toHaveClass("border");
        expect(screen.queryByText("Clear")).not.toBeInTheDocument();
        fireEvent.click(tomePickerBtn);
        expect(screen.getByTestId("composer-tome-picker-popover")).toBeInTheDocument();

        const composerOptions = screen.getAllByTestId("composer-tome-option");
        expect(composerOptions.map((o) => o.textContent)).not.toEqual(
            expect.arrayContaining([expect.stringContaining("Shell Aliases")])
        );

        const searchInput = screen.getByLabelText("Search Eldritch tomes");
        fireEvent.change(searchInput, { target: { value: "File List" } });

        const tomeOptions = screen.getAllByTestId("composer-tome-option");
        expect(tomeOptions).toHaveLength(1);
        fireEvent.click(tomeOptions[0]);

        const scriptTextarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(scriptTextarea.value).toContain("# Tome: File List");
        expect(scriptTextarea.value).toContain('"path": "/tmp"');
        expect(scriptTextarea.value).toContain('file.list(input_params["path"])');
        expect(scriptTextarea.value).not.toContain("process.list()");

        // SessionSettings drawer renders title "Settings", radio toggle & Font Size in the RIGHT panel so text editors stay aligned
        expect(screen.getByTestId("session-settings-drawer")).toBeInTheDocument();
        expect(screen.getByText("Settings")).toBeInTheDocument();
        expect(screen.queryByTestId("rc-eldritch-lsp-status")).not.toBeInTheDocument();
        const rightSettingsPanel = screen.getByTestId("session-settings-right-panel");
        const rcToggle = screen.getByTestId("session-settings-rc-toggle");
        expect(rightSettingsPanel).toContainElement(rcToggle);
        expect(rightSettingsPanel).toHaveTextContent(/Enable RC[\s\S]*Font Size/);
        expect(rcToggle).toHaveAttribute("aria-checked", "false");
        expect(rcToggle).not.toHaveClass("bg-purple-600");

        expect(screen.getByTestId("session-settings-initial-chooser")).toBeInTheDocument();
        const setupAliasesBtn = screen.getByTestId("session-settings-shell-aliases-setup");
        expect(setupAliasesBtn).toHaveTextContent("Set Default Aliases");
        expect(setupAliasesBtn).toHaveClass("bg-purple-600");
        expect(setupAliasesBtn).not.toHaveTextContent(/click setup/i);
        expect(screen.getByTestId("session-settings-load-tome-button")).toHaveTextContent("RC Tome");
        expect(screen.getByTestId("session-settings-load-tome-button")).not.toHaveTextContent(
            "Load RC Tome"
        );
        expect(screen.queryByLabelText("Shared Eldritch RC script editor")).not.toBeInTheDocument();

        // Clicking "Setup Aliases" loads the aliases script, enables RC, and closes the RC window
        fireEvent.click(setupAliasesBtn);
        expect(onCloseSessionSettings).toHaveBeenCalledTimes(1);
        expect(rcToggle).toHaveAttribute("aria-checked", "true");
        expect(rcToggle).toHaveClass("bg-purple-600");
        const rcTextarea = screen.getByLabelText("Shared Eldritch RC script editor") as HTMLTextAreaElement;
        expect(rcTextarea.value).toContain("def ls(");
        expect(rcTextarea.value).toContain("def whoami(");
        expect(rcTextarea.value).toContain("def ps(");
        expect(rcTextarea.value).toContain("def netstat(");

        // Open SHELL_RC Tome picker in SessionSettings -> only shows SHELL_RC tomes ("Shell Aliases")
        fireEvent.click(screen.getByTestId("session-settings-tome-picker-button"));
        expect(screen.getByTestId("session-settings-tome-picker-popover")).toBeInTheDocument();
        const rcTomeOptions = screen.getAllByTestId("session-settings-tome-option");
        expect(rcTomeOptions).toHaveLength(1);
        expect(rcTomeOptions[0]).toHaveTextContent("Shell Aliases");
        fireEvent.click(rcTomeOptions[0]);

        const diagnostics = analyzeEldritchCode(normalizeMultilineScript(rcTextarea.value));
        expect(diagnostics).toHaveLength(0);
    });

    it("places inline font size numeric input with vertical +/- buttons on the right settings panel between min 6 and max 28", () => {
        window.localStorage.clear();
        resetSharedDrawerHeightForTests();
        expect(MIN_SHELL_FONT_SIZE).toBe(6);
        expect(MAX_SHELL_FONT_SIZE).toBe(28);
        expect(clampShellFontSize(2)).toBe(6);
        expect(clampShellFontSize(40)).toBe(28);
        expect(loadShellFontSize()).toBe(DEFAULT_SHELL_FONT_SIZE);

        const StatefulFontSizeShellUI = () => {
            const [fontSize, setFontSize] = React.useState(() => loadShellFontSize());
            const [code, setCode] = React.useState("file.list('/tmp')");
            const termRef = React.useRef<HTMLDivElement>(null);
            return (
                <div className="shell-font-onest">
                    <ShellTerminal
                        termRef={termRef}
                        fontSize={fontSize}
                        isConnecting={false}
                        isPromptBusy={true}
                        promptPos={{ top: 6, left: 6, width: 24, height: 18, visible: true }}
                        completions={[]}
                        showCompletions={false}
                        completionPos={{ x: 0, y: 0 }}
                        completionIndex={0}
                        onMouseMove={vi.fn()}
                        tooltipState={{ visible: false, x: 0, y: 0, signature: "", description: "" }}
                        handleCompletionSelect={vi.fn()}
                    />
                    <MultiLineComposer
                        isOpen={true}
                        onToggle={vi.fn()}
                        code={code}
                        onChange={setCode}
                        onRunCode={vi.fn()}
                        fontSize={fontSize}
                    />
                    <SessionSettings
                        isOpen={true}
                        rcCode={DEFAULT_RC_ALIASES_SCRIPT}
                        fontSize={fontSize}
                        onFontSizeChange={setFontSize}
                        tomes={sampleTomes}
                    />
                </div>
            );
        };

        render(<StatefulFontSizeShellUI />);

        const rightPanel = screen.getByTestId("session-settings-right-panel");
        const fontSizeControl = screen.getByTestId("session-settings-font-size-control");
        const fontSizeInput = screen.getByTestId("session-settings-font-size-input") as HTMLInputElement;
        const decreaseBtn = screen.getByTestId("session-settings-font-decrease");
        const increaseBtn = screen.getByTestId("session-settings-font-increase");

        expect(rightPanel).toContainElement(fontSizeControl);
        expect(fontSizeControl).toContainElement(fontSizeInput);
        expect(fontSizeControl).toContainElement(increaseBtn);
        expect(fontSizeControl).toContainElement(decreaseBtn);
        expect(decreaseBtn).toHaveAttribute("aria-label", "Decrease font size");
        expect(increaseBtn).toHaveAttribute("aria-label", "Increase font size");
        expect(fontSizeInput.value).toBe(String(DEFAULT_SHELL_FONT_SIZE));

        const rcEditor = screen.getByLabelText(
            "Shared Eldritch RC script editor"
        ) as HTMLTextAreaElement;
        const composerEditor = screen.getByLabelText(
            "Multi-line Eldritch script editor"
        ) as HTMLTextAreaElement;
        const promptSpinner = screen.getByTestId("shell-prompt-spinner");

        // Config panel appears AFTER the editor pane in DOM order so editors stay left-aligned
        expect(
            rcEditor.compareDocumentPosition(rightPanel) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();

        expect(rcEditor.style.fontSize).toBe(`${DEFAULT_SHELL_FONT_SIZE}px`);
        expect(composerEditor.style.fontSize).toBe(`${DEFAULT_SHELL_FONT_SIZE}px`);
        expect((promptSpinner.lastElementChild as HTMLElement).style.fontSize).toBe(
            `${DEFAULT_SHELL_FONT_SIZE}px`
        );

        // Increase font size up to MAX_SHELL_FONT_SIZE (28)
        for (let size = DEFAULT_SHELL_FONT_SIZE; size < MAX_SHELL_FONT_SIZE; size++) {
            expect(increaseBtn).not.toBeDisabled();
            fireEvent.click(increaseBtn);
        }
        expect(rcEditor.style.fontSize).toBe("28px");
        expect(composerEditor.style.fontSize).toBe("28px");
        expect((promptSpinner.lastElementChild as HTMLElement).style.fontSize).toBe("28px");
        expect(loadShellFontSize()).toBe(28);
        expect(increaseBtn).toBeDisabled();

        // Decrease font size down to MIN_SHELL_FONT_SIZE (6)
        for (let size = MAX_SHELL_FONT_SIZE; size > MIN_SHELL_FONT_SIZE; size--) {
            expect(decreaseBtn).not.toBeDisabled();
            fireEvent.click(decreaseBtn);
        }
        expect(rcEditor.style.fontSize).toBe("6px");
        expect(composerEditor.style.fontSize).toBe("6px");
        expect((promptSpinner.lastElementChild as HTMLElement).style.fontSize).toBe("6px");
        expect(loadShellFontSize()).toBe(6);
        expect(decreaseBtn).toBeDisabled();

        // Typing directly into the inline numeric input updates font size
        fireEvent.change(fontSizeInput, { target: { value: "18" } });
        expect(rcEditor.style.fontSize).toBe("18px");
        expect(composerEditor.style.fontSize).toBe("18px");
        expect(loadShellFontSize()).toBe(18);
    });

    it("renders composer action buttons in Run -> Save Tome -> Load Tome -> Close order and guides user through creating a Tome with author from AuthorizationContext", async () => {
        const initialScript = [
            "# Tome: File List",
            "# List files and directories with metadata.",
            "input_params = {",
            '    "path": "/tmp",  # Target directory',
            "}",
            "",
            'for f in file.list(input_params["path"]):',
            '    print(f["file_name"], input_params["filter_ext"])',
        ].join("\n");

        const parsed = parseComposerScriptForTome(initialScript, sampleTomes);
        expect(parsed.suggestedName).toBe("File List (Custom)");
        expect(parsed.suggestedDescription).toBe("List files and directories with metadata.");
        expect(parsed.suggestedTactic).toBe("DISCOVERY");
        expect(parsed.cleanedEldritch).toBe(
            'for f in file.list(input_params["path"]):\n    print(f["file_name"], input_params["filter_ext"])'
        );
        expect(parsed.detectedParams).toEqual([
            {
                name: "path",
                label: "Target directory",
                type: "string",
                placeholder: "/tmp",
            },
            {
                name: "filter_ext",
                label: "Filter ext",
                type: "string",
                placeholder: "",
            },
        ]);

        const onTomeCreated = vi.fn();
        const createTomeMock = {
            request: {
                query: CREATE_TOME_MUTATION,
                variables: {
                    input: {
                        name: "Custom File Finder",
                        description: "List files matching extension.",
                        author: "alice",
                        tactic: "RECON",
                        supportModel: "COMMUNITY",
                        paramDefs: JSON.stringify([
                            {
                                name: "path",
                                label: "Target directory",
                                type: "string",
                                placeholder: "/var/log",
                            },
                            {
                                name: "filter_ext",
                                label: "Extension filter",
                                type: "string",
                                placeholder: ".log",
                            },
                        ]),
                        eldritch:
                            'for f in file.list(input_params["path"]):\n    print(f["file_name"], input_params["filter_ext"])',
                    },
                },
            },
            result: {
                data: {
                    createTome: {
                        id: "tome-new-99",
                        name: "Custom File Finder",
                        description: "List files matching extension.",
                        author: "alice",
                        tactic: "RECON",
                        supportModel: "COMMUNITY",
                        paramDefs: JSON.stringify([
                            {
                                name: "path",
                                label: "Target directory",
                                type: "string",
                                placeholder: "/var/log",
                            },
                            {
                                name: "filter_ext",
                                label: "Extension filter",
                                type: "string",
                                placeholder: ".log",
                            },
                        ]),
                        eldritch:
                            'for f in file.list(input_params["path"]):\n    print(f["file_name"], input_params["filter_ext"])',
                    },
                },
            },
        };

        const getShellTomesMock = {
            request: {
                query: GET_SHELL_TOMES,
            },
            result: {
                data: {
                    tomes: {
                        edges: [],
                    },
                },
            },
        };

        const StatefulSaveTomeComposer = () => {
            const [code, setCode] = React.useState(initialScript);
            return (
                <AuthorizationContext.Provider
                    value={{
                        data: {
                            me: {
                                id: "u-alice",
                                name: "alice",
                                photoURL: "",
                                isActivated: true,
                                isAdmin: true,
                            },
                        },
                        isLoading: false,
                        error: undefined,
                    }}
                >
                    <MockedProvider mocks={[createTomeMock, getShellTomesMock]} addTypename={false}>
                        <MultiLineComposer
                            isOpen={true}
                            onToggle={vi.fn()}
                            code={code}
                            onChange={setCode}
                            onRunCode={vi.fn()}
                            tomes={sampleTomes}
                            onTomeCreated={onTomeCreated}
                        />
                    </MockedProvider>
                </AuthorizationContext.Provider>
            );
        };

        render(<StatefulSaveTomeComposer />);

        const runBtn = screen.getByRole("button", { name: /^Run$/i });
        const saveTomeBtn = screen.getByTestId("composer-save-tome-button");
        const tomePickerBtn = screen.getByTestId("composer-tome-picker-button");
        const closeBtn = screen.getByLabelText("Close composer");
        expect(saveTomeBtn).toBeInTheDocument();
        expect(saveTomeBtn.textContent).toBe("");
        expect(screen.getByTestId("composer-save-tome-plus-icon")).toBeInTheDocument();
        // Verify exact order: Run -> Save Tome -> Load Tome -> Close
        expect(
            runBtn.compareDocumentPosition(saveTomeBtn) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(
            saveTomeBtn.compareDocumentPosition(tomePickerBtn) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        expect(
            tomePickerBtn.compareDocumentPosition(closeBtn) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();

        // Click Save Tome button -> opens modal at Step 1 (Details) with vertical FormSteps progression
        fireEvent.click(saveTomeBtn);
        expect(screen.getByTestId("save-tome-modal")).toBeInTheDocument();
        expect(screen.getByRole("navigation", { name: "Progress" })).toBeInTheDocument();
        expect(screen.getByText("Configure details")).toBeInTheDocument();
        expect(screen.getByText("Define parameters")).toBeInTheDocument();
        expect(screen.getByText("Confirm tome")).toBeInTheDocument();
        expect(screen.getByTestId("save-tome-details-step")).toBeInTheDocument();

        const nameInput = screen.getByLabelText(/Tome Name/i) as HTMLInputElement;
        const descInput = screen.getByLabelText(/Description/i) as HTMLTextAreaElement;
        const tacticSelect = screen.getByLabelText(/Tactic/i) as HTMLSelectElement;
        const authorInput = screen.getByLabelText(/Author/i) as HTMLInputElement;

        expect(nameInput.value).toBe("File List (Custom)");
        expect(descInput.value).toBe("List files and directories with metadata.");
        expect(tacticSelect.value).toBe("DISCOVERY");
        expect(tacticSelect).toHaveClass("appearance-none");
        expect(Array.from(tacticSelect.options).map((o) => o.text)).toContain("Recon");
        expect(Array.from(tacticSelect.options).some((o) => o.text.includes("RECON"))).toBe(false);
        expect(authorInput.value).toBe("alice");

        // Duplicate name check against existing tome
        fireEvent.change(nameInput, { target: { value: "File List" } });
        expect(screen.getByTestId("save-tome-duplicate-error")).toBeInTheDocument();
        expect(screen.getByTestId("save-tome-next-button")).toBeDisabled();

        // Update details to valid values
        fireEvent.change(nameInput, { target: { value: "Custom File Finder" } });
        fireEvent.change(descInput, { target: { value: "List files matching extension." } });
        fireEvent.change(tacticSelect, { target: { value: "RECON" } });
        expect(screen.queryByTestId("save-tome-duplicate-error")).not.toBeInTheDocument();

        // Advance to Step 2 (Parameters)
        fireEvent.click(screen.getByTestId("save-tome-next-button"));
        expect(screen.getByTestId("save-tome-params-step")).toBeInTheDocument();

        const paramRows = screen.getAllByTestId("save-tome-param-row");
        expect(paramRows).toHaveLength(2);

        const placeholders = screen.getAllByLabelText("Parameter placeholder") as HTMLInputElement[];
        const labels = screen.getAllByLabelText("Parameter label") as HTMLInputElement[];
        fireEvent.change(placeholders[0], { target: { value: "/var/log" } });
        fireEvent.change(labels[1], { target: { value: "Extension filter" } });
        fireEvent.change(placeholders[1], { target: { value: ".log" } });

        // Advance to Step 3 (Review & Save)
        fireEvent.click(screen.getByTestId("save-tome-next-button"));
        expect(screen.getByTestId("save-tome-review-step")).toBeInTheDocument();
        expect(screen.getByTestId("save-tome-lsp-status")).toHaveTextContent("Syntax OK");

        const scriptArea = screen.getByLabelText("Tome Eldritch Script") as HTMLTextAreaElement;
        expect(scriptArea.value).toBe(
            'for f in file.list(input_params["path"]):\n    print(f["file_name"], input_params["filter_ext"])'
        );

        // Submit Create Tome
        fireEvent.click(screen.getByTestId("save-tome-submit-button"));

        await waitFor(() => {
            expect(onTomeCreated).toHaveBeenCalledWith(
                expect.objectContaining({
                    id: "tome-new-99",
                    name: "Custom File Finder",
                    tactic: "RECON",
                })
            );
        });

        // Newly saved tome is immediately available in the Composer Tome Picker
        fireEvent.click(screen.getByTestId("composer-tome-picker-button"));
        expect(screen.getByText("Custom File Finder")).toBeInTheDocument();
    });

    it("supports Undo/Redo shortcuts (Ctrl+Z, Cmd+Z, Ctrl+Shift+Z, Ctrl+Y) and eldritch-lsp quick fixes", () => {
        const StatefulComposer = () => {
            const [code, setCode] = React.useState("os_info = sys.get_os()");
            return (
                <MultiLineComposer
                    isOpen={true}
                    onToggle={vi.fn()}
                    code={code}
                    onChange={setCode}
                    onRunCode={vi.fn()}
                />
            );
        };

        render(<StatefulComposer />);

        // Should flag deprecated sys.get_os() via eldritch-lsp on the bottom bar (not top bar) with colorless button
        expect(screen.getByTestId("composer-diagnostics-list")).toBeInTheDocument();
        const quickFixBtn = screen.getByText("Replace with sys.info");
        expect(quickFixBtn).not.toHaveClass("bg-amber-500/20");

        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;

        // Hovering over the errored code displays the lint popover
        expect(screen.queryByTestId("composer-lint-popover")).not.toBeInTheDocument();
        fireEvent.mouseMove(textarea);
        const lintPopover = screen.getByTestId("composer-lint-popover");
        expect(lintPopover).toBeInTheDocument();
        expect(lintPopover).toHaveTextContent("sys.get_os");

        fireEvent.click(quickFixBtn);
        expect(textarea.value).toBe("os_info = sys.info()");

        // Undo via Ctrl+Z (Linux/Windows) or Cmd+Z (macOS)
        fireEvent.keyDown(textarea, { key: "z", metaKey: true });
        expect(textarea.value).toBe("os_info = sys.get_os()");

        // Redo via Ctrl+Shift+Z / Cmd+Shift+Z
        fireEvent.keyDown(textarea, { key: "z", metaKey: true, shiftKey: true });
        expect(textarea.value).toBe("os_info = sys.info()");
    });

    it("provides VSCode-style inline ghost autocomplete (Tab accepts immediately, ArrowDown/Up cycles) and indentation linting via eldritch-lsp", () => {
        const completions = completeEldritchCode("file.", 5);
        expect(completions).not.toBeNull();
        expect(completions!.items.some((c) => c.label === "file.list")).toBe(true);

        const badIndentDiagnostics = analyzeEldritchCode("def foo():\nprint(1)");
        expect(badIndentDiagnostics.some((d) => d.message.includes("Expected an indented block"))).toBe(true);

        // `!command` shell macros do not trigger false-positive syntax diagnostics or completions
        expect(analyzeEldritchCode("!ls -la /tmp\nif True:\n    !awk '{print $1}' /etc/passwd")).toEqual([]);
        expect(completeEldritchCode("!file.", 6)).toBeNull();

        const StatefulAutocompleteComposer = () => {
            const [code, setCode] = React.useState("");
            return (
                <MultiLineComposer
                    isOpen={true}
                    onToggle={vi.fn()}
                    code={code}
                    onChange={setCode}
                    onRunCode={vi.fn()}
                />
            );
        };

        render(<StatefulAutocompleteComposer />);
        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;

        // Type "file." at cursor offset 5
        fireEvent.change(textarea, {
            target: { value: "file.", selectionStart: 5, selectionEnd: 5 },
        });
        // Renders inline ghost completion inside the syntax highlight overlay (no redundant bottom bar)
        expect(screen.getByTestId("composer-inline-completion")).toBeInTheDocument();
        expect(screen.queryByTestId("composer-autocomplete-list")).not.toBeInTheDocument();

        const firstOption = completions!.items[0];
        const secondOption = completions!.items[1];

        // ArrowDown cycles to second option, Tab immediately accepts the inline completion (like VSCode)
        fireEvent.keyDown(textarea, { key: "ArrowDown" });
        fireEvent.keyDown(textarea, { key: "Tab" });

        expect(textarea.value).toBe(`file.${secondOption.insertText}`);
        expect(firstOption.insertText).not.toBe(secondOption.insertText);
    });

    it("dynamically derives modules, functions, builtins, value methods, deprecations, and return-type completions from Rust Eldritch sources", () => {
        // Active modules exclude deprecated compatibility modules (crypto, pivot), while allModules includes them
        expect(Object.keys(ELDRITCH_MODULES)).toContain("file");
        expect(Object.keys(ELDRITCH_MODULES)).toContain("process");
        expect(Object.keys(ELDRITCH_MODULES)).toContain("linux");
        expect(Object.keys(ELDRITCH_MODULES)).not.toContain("crypto");
        expect(Object.keys(ELDRITCH_MODULES)).not.toContain("pivot");
        expect(stdlibRegistry.allModules.has("crypto")).toBe(true);
        expect(stdlibRegistry.allModules.has("pivot")).toBe(true);

        // Functions extracted directly from Rust #[eldritch_method] traits (including ones missing from markdown docs)
        const sigSym = stdlibRegistry.functions.get("process.signal");
        expect(sigSym).toBeDefined();
        expect(sigSym!.minArgs).toBe(2);
        expect(sigSym!.maxArgs).toBe(2);

        const resetSym = stdlibRegistry.functions.get("agent.reset_transport");
        expect(resetSym).toBeDefined();
        expect(resetSym!.minArgs).toBe(0);

        // Builtins, keywords, and booleans extracted from eldritch-core
        expect(ELDRITCH_BUILTINS.eval).toBeDefined();
        expect(ELDRITCH_BUILTINS.builtins).toBeDefined();
        expect(ELDRITCH_KEYWORDS).toContain("def");
        expect(ELDRITCH_KEYWORDS).toContain("lambda");
        expect(ELDRITCH_BOOLEANS).toEqual(expect.arrayContaining(["True", "False", "None"]));

        // Deprecations extracted from #[eldritch_deprecated] and migration rules
        expect(DEPRECATED_STDLIB_FUNCTIONS["sys.get_os"]).toBe("sys.info");
        expect(DEPRECATED_STDLIB_FUNCTIONS["crypto.md5"]).toBe("hash.md5");
        expect(DEPRECATED_STDLIB_FUNCTIONS["process.netstat"]).toBe("net.conns");

        // Dynamic return-type inference for value methods (Bytes, Dict, str, List)
        const bytesComp = completeEldritchCode('raw = assets.read("bin")\nraw.', 29);
        expect(bytesComp).not.toBeNull();
        expect(bytesComp!.items.some((i) => i.label === "bytes.decode")).toBe(true);

        const dictComp = completeEldritchCode("u = linux.uname()\nu.", 20);
        expect(dictComp).not.toBeNull();
        expect(dictComp!.items.some((i) => i.label === "dict.keys")).toBe(true);

        const strComp = completeEldritchCode("s = random.uuid()\ns.", 20);
        expect(strComp).not.toBeNull();
        expect(strComp!.items.some((i) => i.label === "str.encode")).toBe(true);
    });

    it("registers and autocompletes user-defined functions and variables (such as RC aliases)", () => {
        clearUserDefinedSymbols();

        // 1. Initially no user defined functions or variables
        expect(getUserDefinedFunctions()).toHaveLength(0);
        expect(getUserDefinedVariables()).toHaveLength(0);

        // 2. Parse and register DEFAULT_RC_ALIASES_SCRIPT (simulating RC load)
        const { functions } = registerUserSymbolsFromCode(DEFAULT_RC_ALIASES_SCRIPT);
        expect(functions.map((f) => f.name)).toEqual(
            expect.arrayContaining(["ls", "whoami", "ps", "pwd", "cat", "netstat"])
        );

        // 3. Autocomplete user-defined functions
        const lsComp = completeEldritchCode("l", 1);
        expect(lsComp).not.toBeNull();
        const lsItem = lsComp!.items.find((i) => i.label === "ls");
        expect(lsItem).toBeDefined();
        expect(lsItem!.kind).toBe("function");
        expect(lsItem!.detail).toContain('ls(path=".")');

        const whoamiComp = completeEldritchCode("who", 3);
        expect(whoamiComp).not.toBeNull();
        expect(whoamiComp!.items.some((i) => i.label === "whoami")).toBe(true);

        // 4. Parameter completions and signature help on user-defined function
        const paramComp = completeEldritchCode("ls(p", 4);
        expect(paramComp).not.toBeNull();
        expect(paramComp!.items.some((i) => i.label === "path=")).toBe(true);

        const sigHelp = getEldritchSignatureHelp("ls(", 3);
        expect(sigHelp).not.toBeNull();
        expect(sigHelp!.functionName).toBe("ls");

        // 5. Register and autocomplete user-defined variables with type inference
        registerUserSymbolsFromCode('TARGET_HOST = "10.10.10.5"\nPORT_LIST = [80, 443]\nAGENT_CONFIG = {"active": True}');
        expect(getUserDefinedVariables().map((v) => v.name)).toEqual(
            expect.arrayContaining(["TARGET_HOST", "PORT_LIST", "AGENT_CONFIG"])
        );

        const targetComp = completeEldritchCode("TAR", 3);
        expect(targetComp).not.toBeNull();
        expect(targetComp!.items.some((i) => i.label === "TARGET_HOST")).toBe(true);

        // 6. Inferred type method completion on user variables (dict methods, list methods, str methods)
        const agentConfigMethods = completeEldritchCode("AGENT_CONFIG.", 13);
        expect(agentConfigMethods).not.toBeNull();
        expect(agentConfigMethods!.items.some((i) => i.label === "dict.keys")).toBe(true);
        expect(agentConfigMethods!.items.some((i) => i.label === "dict.values")).toBe(true);

        const hostMethods = completeEldritchCode("TARGET_HOST.", 12);
        expect(hostMethods).not.toBeNull();
        expect(hostMethods!.items.some((i) => i.label === "str.encode")).toBe(true);

        // 7. BrowserWasmAdapter completes registered user functions and variables
        const adapter = new BrowserWasmAdapter(
            "ws://localhost:8080/ws",
            () => {},
            () => {},
            () => {}
        );
        const adapterRes = adapter.complete("who", 3);
        expect(adapterRes.suggestions).toContain("whoami");

        const adapterVarRes = adapter.complete("TAR", 3);
        expect(adapterVarRes.suggestions).toContain("TARGET_HOST");

        // 8. Dynamic clearing of user symbols
        clearUserDefinedSymbols();
        expect(getUserDefinedFunctions()).toHaveLength(0);
        expect(getUserDefinedVariables()).toHaveLength(0);
        const afterClearComp = completeEldritchCode("who", 3);
        expect(afterClearComp?.items.some((i) => i.label === "whoami")).toBeFalsy();
    });

    it("completes user-defined functions on Tab with function call syntax and parameter completions", () => {
        clearUserDefinedSymbols();

        // Register default aliases script containing ls(path="."), cat(path), whoami()
        registerUserSymbolsFromCode(DEFAULT_RC_ALIASES_SCRIPT);

        // Verify minArgs computation:
        // whoami has 0 params -> minArgs 0
        const whoamiSym = stdlibRegistry.userFunctions.get("whoami");
        expect(whoamiSym).toBeDefined();
        expect(whoamiSym!.minArgs).toBe(0);

        // ls has path="." (with default) -> minArgs 0
        const lsSym = stdlibRegistry.userFunctions.get("ls");
        expect(lsSym).toBeDefined();
        expect(lsSym!.minArgs).toBe(0);

        // cat has path (no default) -> minArgs 1
        const catSym = stdlibRegistry.userFunctions.get("cat");
        expect(catSym).toBeDefined();
        expect(catSym!.minArgs).toBe(1);

        // Check completeEldritchCode on exact function names and prefixes
        const whoComp = completeEldritchCode("who", 3);
        expect(whoComp?.items.some((i) => i.label === "whoami")).toBe(true);

        const whoamiExactComp = completeEldritchCode("whoami", 6);
        expect(whoamiExactComp?.items.some((i) => i.label === "whoami")).toBe(true);

        // Check parameter completion for cat
        const catParamComp = completeEldritchCode("cat(", 4);
        expect(catParamComp?.items.some((i) => i.label === "path=")).toBe(true);

        // Check parameter completion with prefix
        const catPComp = completeEldritchCode("cat(p", 5);
        expect(catPComp?.items.some((i) => i.label === "path=")).toBe(true);

        // Register a custom function with multiple params (some default, some required)
        registerUserSymbolsFromCode("def deploy_payload(host, port=22, timeout=60):\n    return True");
        const deploySym = stdlibRegistry.userFunctions.get("deploy_payload");
        expect(deploySym).toBeDefined();
        expect(deploySym!.minArgs).toBe(1); // 'host' is required, 'port' and 'timeout' are optional

        const deployParamComp = completeEldritchCode("deploy_payload(", 15);
        expect(deployParamComp?.items.map((i) => i.label)).toEqual(
            expect.arrayContaining(["host=", "port=", "timeout="])
        );
    });
});

describe("ShellHistoryDrawer & ShellPivotsDrawer", () => {
    const sampleTasks: ShellTaskItem[] = [
        {
            id: "task-1",
            createdAt: "2025-01-01T12:00:00Z",
            claimedAt: "2025-01-01T12:00:01Z",
            execStartedAt: "2025-01-01T12:00:01Z",
            execFinishedAt: "2025-01-01T12:00:02Z",
            input: 'report.user_password("admin", "hunter2")',
            output: "Reported credential",
            creator: { id: "u1", name: "operator1" },
            reportedCredentials: [{ id: "c1", principal: "admin", kind: "KIND_PASSWORD" }],
        },
    ];

    const samplePivots: ShellPivotItem[] = [
        {
            id: "42",
            kind: "SSH",
            destination: "root@10.0.0.15:22",
            closedAt: "2025-01-01T12:10:00Z",
        },
    ];

    it("displays History drawer titled 'History' and renders pivots in dedicated ShellPivotsDrawer with PTY and SSH buttons", () => {
        const onRunCode = vi.fn();
        const onLoadIntoComposer = vi.fn();
        const onOpenPivotTab = vi.fn();
        const onOpenPty = vi.fn();
        const onOpenSshModal = vi.fn();

        render(
            <>
                <ShellHistoryDrawer
                    isOpen={true}
                    onClose={vi.fn()}
                    shellTasks={sampleTasks}
                    localHistory={[]}
                    onRunCode={onRunCode}
                    onLoadIntoComposer={onLoadIntoComposer}
                />
                <ShellPivotsDrawer
                    isOpen={true}
                    onClose={vi.fn()}
                    portalId={99}
                    pivots={samplePivots}
                    onNewPortal={vi.fn()}
                    onClosePortal={vi.fn()}
                    onOpenPty={onOpenPty}
                    onOpenSshModal={onOpenSshModal}
                    onOpenPivotTab={onOpenPivotTab}
                />
            </>
        );

        expect(screen.getByText("History")).toBeInTheDocument();
        expect(screen.getByRole("tab", { name: /Creds \(1\)/i })).toBeInTheDocument();

        // Switch to Output tab
        fireEvent.click(screen.getByRole("tab", { name: /Output/i }));
        expect(screen.getByText("Reported credential")).toBeInTheDocument();

        // Switch to Creds tab
        fireEvent.click(screen.getByRole("tab", { name: /Creds \(1\)/i }));
        expect(screen.getByText("admin")).toBeInTheDocument();

        // Action buttons are icon-only (no text labels) for Run, Copy, and Edit
        expect(screen.queryByText("Re-run")).not.toBeInTheDocument();
        expect(screen.queryByText("Composer")).not.toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "Re-run" }));
        expect(onRunCode).toHaveBeenCalledWith('report.user_password("admin", "hunter2")');
        fireEvent.click(screen.getByRole("button", { name: "Edit in Composer" }));
        expect(onLoadIntoComposer).toHaveBeenCalledWith('report.user_password("admin", "hunter2")');

        // Dedicated Pivots drawer displays PTY and SSH buttons and opens saved transcript
        expect(screen.getByTestId("shell-pivots-drawer")).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "PTY" }));
        expect(onOpenPty).toHaveBeenCalledTimes(1);
        fireEvent.click(screen.getByRole("button", { name: "SSH" }));
        expect(onOpenSshModal).toHaveBeenCalledTimes(1);
        expect(screen.getByText(/SSH • root@10.0.0.15:22/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText("View Saved Transcript"));
        expect(onOpenPivotTab).toHaveBeenCalledWith("ssh", "root@10.0.0.15:22", "42");

        // Pivot tab labels omit IDs and show IP for SSH or static "PTY" for PTY (no dynamic command naming)
        expect(formatPivotTabLabel({ type: "ssh", target: "root:secret@10.0.0.15:22" })).toBe("10.0.0.15");
        expect(formatPivotTabLabel({ type: "pty", target: "PTY #90194313217" })).toBe("PTY");
        expect(formatPivotTabLabel({ type: "pty", target: "exit" })).toBe("PTY");
        expect(
            formatPivotTabLabel({
                type: "pty",
                target: "cat /etc/shadow && grep root /etc/passwd",
            })
        ).toBe("PTY");

        // Manual rename overrides default naming for PTY and SSH
        expect(
            formatPivotTabLabel({
                type: "pty",
                target: "cat /etc/shadow && grep root /etc/passwd",
                customTitle: "Shadow Dump",
            })
        ).toBe("Shadow Dump");
        expect(
            formatPivotTabLabel({
                type: "ssh",
                target: "root:secret@10.0.0.15:22",
                customTitle: "Bastion Jump",
            })
        ).toBe("Bastion Jump");
    });

    it("strictly filters Queued / Running commands and responds to focusQueueTrigger", () => {
        const activeTask: ShellTaskItem = {
            id: "task-queued",
            createdAt: "2025-01-01T12:05:00Z",
            input: 'sys.shell("sleep 10")',
        };

        const { rerender } = render(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={[...sampleTasks, activeTask]}
                localHistory={['print("old local command")']}
                onRunCode={vi.fn()}
                onLoadIntoComposer={vi.fn()}
                focusQueueTrigger={0}
            />
        );

        // Both tasks visible under All
        expect(screen.getByText('report.user_password("admin", "hunter2")')).toBeInTheDocument();
        expect(screen.getByText('sys.shell("sleep 10")')).toBeInTheDocument();

        // Increment focusQueueTrigger -> switches to Queued / Running
        rerender(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={[...sampleTasks, activeTask]}
                localHistory={['print("old local command")']}
                onRunCode={vi.fn()}
                onLoadIntoComposer={vi.fn()}
                focusQueueTrigger={1}
            />
        );
        expect(screen.getByText('sys.shell("sleep 10")')).toBeInTheDocument();
        expect(screen.queryByText('report.user_password("admin", "hunter2")')).not.toBeInTheDocument();
        expect(screen.queryByText('print("old local command")')).not.toBeInTheDocument();
    });

    it("supports cross-session history toggle, aggregates similar outputs, and truncates long output with a Show full toggle", () => {
        const longOutput = ["line 1", "line 2", "line 3", "line 4", "line 5"].join("\n");
        const currentSessionTasks: ShellTaskItem[] = [
            {
                id: "t-curr-1",
                shellId: "10",
                createdAt: "2025-01-01T12:00:00Z",
                execFinishedAt: "2025-01-01T12:00:01Z",
                input: "whoami()",
                output: "root",
            },
            {
                id: "t-curr-2",
                shellId: "10",
                createdAt: "2025-01-01T12:01:00Z",
                execFinishedAt: "2025-01-01T12:01:01Z",
                input: "whoami()",
                output: "root",
            },
        ];
        const crossSessionTasks: ShellTaskItem[] = [
            ...currentSessionTasks,
            {
                id: "t-other-1",
                shellId: "99",
                createdAt: "2025-01-01T12:02:00Z",
                execFinishedAt: "2025-01-01T12:02:01Z",
                input: "file.list('/etc')",
                output: longOutput,
            },
        ];

        const onSelectShellMock = vi.fn();
        render(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={currentSessionTasks}
                allShellTasks={crossSessionTasks}
                currentShellId="10"
                onRunCode={vi.fn()}
                onLoadIntoComposer={vi.fn()}
                onSelectShell={onSelectShellMock}
            />
        );

        // Two identical whoami() runs are aggregated with a ×2 badge inline to the right of the time
        const aggregatedBadge = screen.getByTestId("history-aggregated-count");
        expect(aggregatedBadge).toHaveTextContent("×2");
        const timeEls = screen.getAllByTestId("history-task-time");
        expect(
            timeEls[0].compareDocumentPosition(aggregatedBadge) & Node.DOCUMENT_POSITION_FOLLOWING
        ).toBeTruthy();
        // Cross-session task displays a condensed terminal icon button to swap shells instead of raw shell id text
        expect(screen.queryByText(/Shell #/i)).not.toBeInTheDocument();
        const swapButton = screen.getByTestId("history-card-swap-shell");
        expect(swapButton).toBeInTheDocument();
        expect(swapButton).toHaveAttribute("title", "Swap to Shell #99");
        fireEvent.click(swapButton);
        expect(onSelectShellMock).toHaveBeenCalledWith("99");

        // Switch to Output tab on the long-output task and verify truncation + Show full toggle
        const outputTabs = screen.getAllByRole("tab", { name: /Output/i });
        fireEvent.click(outputTabs[0]);
        expect(screen.queryByText(/line 5/)).not.toBeInTheDocument();

        const toggleFullBtn = screen.getByTestId("history-toggle-full");
        expect(toggleFullBtn).toHaveTextContent(/Show full/i);
        fireEvent.click(toggleFullBtn);
        expect(screen.getByText(/line 5/)).toBeInTheDocument();

        // Toggle scope to Current Session only -> Shell swap button is hidden
        const scopeToggle = screen.getByTestId("history-session-scope-toggle");
        expect(scopeToggle).toHaveTextContent("All Sessions");
        fireEvent.click(scopeToggle);
        expect(scopeToggle).toHaveTextContent("Current Session");
        expect(screen.queryByTestId("history-card-swap-shell")).not.toBeInTheDocument();
        expect(screen.queryByText(/Shell #/i)).not.toBeInTheDocument();
    });
});

describe("Access Drawer (HostsTreeDrawer) & SshConnectionModal", () => {
    const sampleCreds: KnowledgeCredentialItem[] = [
        {
            id: "cred-1",
            principal: "svc_backup",
            kind: "KIND_PASSWORD",
            secret: "Winter2025!",
            createdAt: "2025-01-01T12:00:00Z",
            hostId: "h-1",
            hostName: "web-01",
            hostPrimaryIP: "10.0.0.10",
            isCurrentHost: true,
        },
    ];

    const sampleHosts: KnowledgeHostItem[] = [
        {
            id: "h-2",
            hostId: "h-2",
            networkHostId: "nh-2",
            name: "db-01",
            hostname: "db-01.corp.local",
            primaryIP: "10.0.0.20",
            ipAddress: "10.0.0.20",
            subnet: "10.0.0.0/24",
            openPorts: [22, 5432],
            platform: "LINUX",
            tags: [{ id: "t1", name: "prod", kind: "GROUP" }],
            beaconCount: 1,
            credentialCount: 1,
            credentials: [],
            isCurrentHost: false,
            sharedTags: ["prod"],
            sameSubnet: true,
        },
    ];

    it("computes unified adjacent hosts, credentials, pivots, shells, and files in networkGraphUtils", () => {
        expect(getIpv4SubnetPrefix("10.0.0.15")).toBe("10.0.0.");
        expect(formatSubnetCidr("10.0.0.15")).toBe("10.0.0.0/24");
        expect(extractPivotTargetHost("root:secret@10.0.0.25:22")).toBe("10.0.0.25");
        expect(formatFileSize(2048)).toBe("2.0 KB");
        expect(parsePortList("22, :443, 80, 22")).toEqual([22, 80, 443]);
        expect(formatPortListInput([22, 443])).toBe("22, 443");
        expect(togglePortInList([22, 5432], 5432)).toEqual([22]);
        expect(togglePortInList([22], 443)).toEqual([22, 443]);

        const result = buildUnifiedGraphData({
            currentHostNode: {
                id: "1",
                name: "jumpbox",
                primaryIP: "10.0.0.5",
                credentials: {
                    edges: [
                        {
                            node: {
                                id: "c-51",
                                principal: "nasadmin",
                                kind: "KIND_PASSWORD",
                                secret: "naspass",
                                createdAt: "2025-01-01T00:00:00Z",
                            },
                        },
                    ],
                },
                files: { edges: [] },
                beacons: { edges: [] },
            },
            hostId: "1",
            currentShellId: "10",
            currentHostTags: [],
            currentShellPivots: [
                {
                    id: "p-1",
                    kind: "ssh",
                    destination: "root@10.0.0.99:22",
                },
            ],
            knowledgeData: {
                hosts: {
                    edges: [
                        {
                            node: {
                                id: "2",
                                name: "internal-db",
                                primaryIP: "10.0.0.99",
                                credentials: {
                                    edges: [
                                        {
                                            node: {
                                                id: "c-99",
                                                principal: "dbadmin",
                                                kind: "KIND_PASSWORD",
                                                secret: "pgpass",
                                                createdAt: "2025-01-01T00:00:00Z",
                                            },
                                        },
                                    ],
                                },
                                files: {
                                    edges: [
                                        {
                                            node: {
                                                id: "f-99",
                                                path: "/var/lib/pgsql/backup.sql",
                                                size: 4096,
                                                createdAt: "2025-01-01T00:00:00Z",
                                            },
                                        },
                                    ],
                                },
                                beacons: {
                                    edges: [
                                        {
                                            node: {
                                                id: "b-99",
                                                name: "db-beacon",
                                                principal: "postgres",
                                                shells: {
                                                    edges: [
                                                        {
                                                            node: {
                                                                id: "20",
                                                                createdAt: "2025-01-01T00:00:00Z",
                                                                closedAt: null,
                                                                pivots: { edges: [] },
                                                            },
                                                        },
                                                    ],
                                                },
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                    ],
                },
                networkHosts: {
                    edges: [
                        {
                            node: {
                                id: "50",
                                name: "internal-db",
                                hostname: "internal-db.local",
                                ipAddress: "10.0.0.99",
                                subnet: "10.0.0.0/24",
                                openPorts: [22, 5432],
                                host: { id: "2", name: "internal-db", primaryIP: "10.0.0.99" },
                                networks: {
                                    edges: [
                                        {
                                            node: {
                                                id: "net-1",
                                                name: "10.0.0.0/24",
                                                subnet: "10.0.0.0/24",
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                        {
                            node: {
                                id: "51",
                                name: "discovered-nas",
                                hostname: "nas.local",
                                ipAddress: "10.0.0.88",
                                subnet: "10.0.0.0/24",
                                openPorts: [22, 445],
                                host: null,
                                credentials: {
                                    edges: [
                                        {
                                            node: {
                                                id: "c-51",
                                                principal: "nasadmin",
                                                kind: "KIND_PASSWORD",
                                                secret: "naspass",
                                                createdAt: "2025-01-01T00:00:00Z",
                                            },
                                        },
                                    ],
                                },
                                networks: { edges: [] },
                            },
                        },
                        {
                            node: {
                                id: "52",
                                name: "unreachable-isolated",
                                hostname: "isolated.local",
                                ipAddress: "192.168.99.99",
                                subnet: "192.168.99.0/24",
                                openPorts: [80],
                                host: null,
                                credentials: { edges: [] },
                                networks: { edges: [] },
                            },
                        },
                    ],
                },
            },
        });

        expect(result.networkHosts).toHaveLength(3);
        expect(result.networkHosts.some((h) => h.name === "unreachable-isolated")).toBe(true);
        // Same-subnet hosts with open ports sort ahead of isolated hosts on other subnets
        expect(result.networkHosts[2].name).toBe("unreachable-isolated");
        const internalDbHost = result.networkHosts.find((h) => h.name === "internal-db");
        expect(internalDbHost?.openPorts).toEqual([22, 5432]);
        expect(internalDbHost?.hostId).toBe("2");
        expect(result.allCredentials).toHaveLength(2);
        expect(result.allCredentials.some((c) => c.principal === "dbadmin")).toBe(true);
        const nasCred = result.allCredentials.find((c) => c.id === "c-51");
        expect(nasCred?.hostName).toBe("discovered-nas");
        expect(nasCred?.hostPrimaryIP).toBe("10.0.0.88");
        expect(nasCred?.isCurrentHost).toBe(false);
        expect(result.hostCredentials).toHaveLength(0);
        expect(result.hostFiles).toHaveLength(1);
        expect(result.hostFiles[0].path).toBe("/var/lib/pgsql/backup.sql");
        expect(result.hostShells).toHaveLength(1);
        expect(result.hostShells[0].id).toBe("20");
        expect(result.nearbyPivots).toHaveLength(1);
    });

    it("populates SshConnectionModal target from Quick Fill hosts and credentials, filters creds by selected host, removes Port field, and saves inline + credentials", async () => {
        const onConnect = vi.fn();
        const onLoadIntoComposer = vi.fn();
        const onCredentialCreated = vi.fn();
        const credsWithSshKey: KnowledgeCredentialItem[] = [
            ...sampleCreds,
            {
                id: "cred-ssh-key-2",
                principal: "deploy",
                kind: "KIND_SSH_KEY",
                secret: "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmU=\n-----END OPENSSH PRIVATE KEY-----",
                createdAt: "2025-01-01T12:00:00Z",
                hostId: "h-2",
                hostName: "db-01",
                hostPrimaryIP: "10.0.0.20",
                isCurrentHost: false,
            },
        ];

        const createInlineCredMock = {
            request: {
                query: CREATE_CREDENTIAL_MUTATION,
                variables: {
                    input: {
                        principal: "dbops",
                        secret: "InlinePass2026!",
                        kind: "KIND_PASSWORD",
                        networkHostIDs: ["nh-2"],
                        hostID: "h-2",
                    },
                },
            },
            result: {
                data: {
                    createCredential: {
                        id: "cred-inline-saved",
                        principal: "dbops",
                        kind: "KIND_PASSWORD",
                        secret: "InlinePass2026!",
                        createdAt: "2026-01-01T00:00:00Z",
                    },
                },
            },
        };

        render(
            <MockedProvider mocks={[createInlineCredMock]} addTypename={false}>
                <SshConnectionModal
                    isOpen={true}
                    onClose={vi.fn()}
                    onConnect={onConnect}
                    credentials={credsWithSshKey}
                    hosts={sampleHosts}
                    onCredentialCreated={onCredentialCreated}
                    onLoadIntoComposer={onLoadIntoComposer}
                />
            </MockedProvider>
        );

        expect(screen.getByRole("heading", { name: "Pivot" })).toBeInTheDocument();
        expect(screen.queryByText("SSH Connection")).not.toBeInTheDocument();
        const pivotTypeSelect = screen.getByLabelText("Pivot Type") as HTMLSelectElement;
        expect(pivotTypeSelect.value).toBe("SSH");
        expect(screen.getByTestId("pivot-modal-available-pivots")).toHaveTextContent(
            /Available Pivots/i
        );
        expect(screen.getByText("Quick Fill")).toBeInTheDocument();
        expect(screen.queryByText("Quick Fill from Knowledge Graph")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
        expect(screen.queryByLabelText(/^Port/i)).not.toBeInTheDocument();

        // Initially both credentials are visible (unfiltered)
        expect(screen.getByTestId("ssh-cred-option-cred-1")).toBeInTheDocument();
        expect(screen.getByTestId("ssh-cred-option-cred-ssh-key-2")).toBeInTheDocument();

        // Select host db-01 (10.0.0.20): cred-1 (web-01) is filtered out while deploy (db-01) remains
        fireEvent.click(screen.getByTestId("ssh-host-option-h-2"));
        expect(screen.queryByTestId("ssh-cred-option-cred-1")).not.toBeInTheDocument();
        expect(screen.getByTestId("ssh-cred-option-cred-ssh-key-2")).toBeInTheDocument();

        // Select deploy credential on db-01
        fireEvent.click(screen.getByTestId("ssh-cred-option-cred-ssh-key-2"));
        const input = screen.getByLabelText("Target") as HTMLInputElement;
        expect(input.value).toBe("deploy@10.0.0.20:22");

        // Add inline credential via + button (no nested modal opened)
        fireEvent.click(screen.getByTestId("ssh-add-credential-button"));
        expect(screen.getByTestId("ssh-inline-credential-editor")).toBeInTheDocument();
        expect(screen.queryByTestId("credential-modal")).not.toBeInTheDocument();

        fireEvent.change(screen.getByLabelText("New credential principal"), {
            target: { value: "dbops" },
        });
        fireEvent.change(screen.getByLabelText("New credential secret"), {
            target: { value: "InlinePass2026!" },
        });
        fireEvent.click(screen.getByTestId("ssh-save-inline-credential"));

        await waitFor(() => {
            expect(onCredentialCreated).toHaveBeenCalledWith(
                expect.objectContaining({ id: "cred-inline-saved", principal: "dbops" }),
                "h-2"
            );
        });
        expect(input.value).toBe("dbops@10.0.0.20:22");

        // Unselect host db-01 to clear filter, verify cred-1 reappears and works with 10.0.0.20
        fireEvent.click(screen.getByTestId("ssh-host-option-h-2"));
        fireEvent.click(screen.getByTestId("ssh-cred-option-cred-1"));
        expect(input.value).toBe("svc_backup@10.0.0.20:22");

        // Re-select SSH key credential deploy and connect
        fireEvent.click(screen.getByTestId("ssh-cred-option-cred-ssh-key-2"));
        expect(input.value).toBe("deploy@10.0.0.20:22");

        fireEvent.click(screen.getByText("Connect"));
        expect(onConnect).toHaveBeenCalledWith("deploy@10.0.0.20:22", "cred-ssh-key-2", "SSH");
    });

    it("does not close the Access drawer when ShellWalkthrough is closed or re-rendered", () => {
        const onCloseAccessDrawer = vi.fn();
        const { rerender } = render(
            <ShellWalkthrough
                isOpen={false}
                onDismiss={vi.fn()}
                onOpenComposer={() => { }}
                onCloseComposer={() => { }}
                onOpenSessionSettings={() => { }}
                onCloseSessionSettings={() => { }}
                onOpenAccessDrawer={() => { }}
                onCloseAccessDrawer={onCloseAccessDrawer}
            />
        );

        expect(onCloseAccessDrawer).not.toHaveBeenCalled();

        // Simulate parent re-render (e.g., clicking Access drawer toggle button) with fresh inline callbacks
        rerender(
            <ShellWalkthrough
                isOpen={false}
                onDismiss={vi.fn()}
                onOpenComposer={() => { }}
                onCloseComposer={() => { }}
                onOpenSessionSettings={() => { }}
                onCloseSessionSettings={() => { }}
                onOpenAccessDrawer={() => { }}
                onCloseAccessDrawer={onCloseAccessDrawer}
            />
        );

        expect(onCloseAccessDrawer).not.toHaveBeenCalled();
    });

    it("supports creating hosts and credentials from the Access drawer and editing discovered hosts via HostModal", async () => {
        const onHostSaved = vi.fn();
        const onCredentialCreated = vi.fn();

        const createNetworkHostMock = {
            request: {
                query: CREATE_NETWORK_HOST_MUTATION,
                variables: {
                    input: {
                        name: "fileserver",
                        hostname: "fileserver",
                        ipAddress: "10.0.0.30",
                        subnet: "10.0.0.0/24",
                        openPorts: [22, 445],
                    },
                },
            },
            result: {
                data: {
                    createNetworkHost: {
                        id: "nh-30",
                        name: "fileserver",
                        hostname: "fileserver",
                        ipAddress: "10.0.0.30",
                        subnet: "10.0.0.0/24",
                        openPorts: [22, 445],
                        host: null,
                        networks: { edges: [] },
                    },
                },
            },
        };

        const updateNetworkHostMock = {
            request: {
                query: UPDATE_NETWORK_HOST_MUTATION,
                variables: {
                    networkHostID: "nh-2",
                    input: {
                        name: "db-01",
                        hostname: "db-01",
                        ipAddress: "10.0.0.20",
                        openPorts: [22, 443],
                        addCredentialIDs: ["cred-1"],
                    },
                },
            },
            result: {
                data: {
                    updateNetworkHost: {
                        id: "nh-2",
                        name: "db-01",
                        hostname: "db-01.corp.local",
                        ipAddress: "10.0.0.20",
                        subnet: "10.0.0.0/24",
                        openPorts: [22, 443],
                    },
                },
            },
        };

        const createCredentialMock = {
            request: {
                query: CREATE_CREDENTIAL_MUTATION,
                variables: {
                    input: {
                        hostID: "h-1",
                        principal: "deploy",
                        secret: "DeployKey2026!",
                        kind: "KIND_PASSWORD",
                    },
                },
            },
            result: {
                data: {
                    createCredential: {
                        id: "cred-99",
                        principal: "deploy",
                        kind: "KIND_PASSWORD",
                        secret: "DeployKey2026!",
                        createdAt: "2026-01-01T00:00:00Z",
                    },
                },
            },
        };

        const createNetworkHostCredMock = {
            request: {
                query: CREATE_CREDENTIAL_MUTATION,
                variables: {
                    input: {
                        networkHostIDs: ["nh-30"],
                        principal: "postgres",
                        secret: "PgPass2026!",
                        kind: "KIND_PASSWORD",
                    },
                },
            },
            result: {
                data: {
                    createCredential: {
                        id: "cred-100",
                        principal: "postgres",
                        kind: "KIND_PASSWORD",
                        secret: "PgPass2026!",
                        createdAt: "2026-01-01T00:00:00Z",
                    },
                },
            },
        };

        const fileserverHost: KnowledgeHostItem = {
            id: "nh-30",
            networkHostId: "nh-30",
            name: "fileserver",
            hostname: "fileserver",
            primaryIP: "10.0.0.30",
            ipAddress: "10.0.0.30",
            subnet: "10.0.0.0/24",
            openPorts: [22, 445],
            tags: [],
            beaconCount: 0,
            credentialCount: 0,
            credentials: [],
            isCurrentHost: false,
            sharedTags: [],
            sameSubnet: true,
        };

        const { rerender } = render(
            <MockedProvider
                mocks={[
                    createNetworkHostMock,
                    updateNetworkHostMock,
                    createCredentialMock,
                    createNetworkHostCredMock,
                ]}
                addTypename={false}
            >
                <BrowserRouter>
                    <HostsTreeDrawer
                        isOpen={true}
                        onClose={vi.fn()}
                        currentShellId="1"
                        currentHostId="h-1"
                        currentHostName="web-01"
                        currentHostIp="10.0.0.10"
                        hosts={sampleHosts}
                        shells={[]}
                        pivots={[]}
                        availableCredentials={sampleCreds}
                        onHostSaved={onHostSaved}
                        onCredentialCreated={onCredentialCreated}
                    />
                </BrowserRouter>
            </MockedProvider>
        );

        // 1. Click Monitor+ button in Access drawer header to create a host
        fireEvent.click(screen.getByTestId("hosts-tree-add-button"));
        expect(screen.getByTestId("host-modal")).toBeInTheDocument();
        expect(screen.getByText("New Host")).toBeInTheDocument();
        const hostSubmitBtn = screen.getByTestId("host-modal-submit");
        expect(hostSubmitBtn).toHaveTextContent("Create");
        expect(hostSubmitBtn).toHaveClass("bg-purple-600");

        fireEvent.change(screen.getByLabelText("IP Address"), {
            target: { value: "10.0.0.30" },
        });
        fireEvent.change(screen.getByLabelText("Name / Hostname"), {
            target: { value: "fileserver" },
        });
        fireEvent.change(screen.getByLabelText("Open Ports"), {
            target: { value: "22, 445" },
        });

        fireEvent.click(screen.getByTestId("host-modal-submit"));

        await waitFor(() => {
            expect(onHostSaved).toHaveBeenCalledWith(
                expect.objectContaining({
                    id: "nh-nh-30",
                    networkHostId: "nh-30",
                    name: "fileserver",
                    ipAddress: "10.0.0.30",
                }),
                "create"
            );
        });

        // 2. Click KeyRound+ button in Access drawer header to create a credential for the current host
        fireEvent.click(screen.getByTestId("hosts-tree-add-credential-button"));
        expect(screen.getByTestId("credential-modal")).toBeInTheDocument();
        expect(screen.getByText("New Credential")).toBeInTheDocument();
        const credSubmitBtn = screen.getByTestId("credential-modal-submit");
        expect(credSubmitBtn).toHaveTextContent("Create");
        expect(credSubmitBtn).toHaveClass("bg-purple-600");
        expect(screen.getByLabelText("Kind / Type")).toHaveClass("appearance-none", "pr-8");
        expect(screen.getByLabelText("Host")).toHaveClass("appearance-none", "pr-8");

        fireEvent.change(screen.getByLabelText("Principal / Username"), {
            target: { value: "deploy" },
        });
        fireEvent.change(screen.getByLabelText("Secret / Password"), {
            target: { value: "DeployKey2026!" },
        });

        fireEvent.click(screen.getByTestId("credential-modal-submit"));

        await waitFor(() => {
            expect(onCredentialCreated).toHaveBeenCalledWith(
                expect.objectContaining({ id: "cred-99", principal: "deploy" }),
                undefined
            );
        });

        // 3. Re-render Access drawer with fileserver in hosts list and create a credential bound to fileserver
        rerender(
            <MockedProvider
                mocks={[
                    createNetworkHostMock,
                    updateNetworkHostMock,
                    createCredentialMock,
                    createNetworkHostCredMock,
                ]}
                addTypename={false}
            >
                <BrowserRouter>
                    <HostsTreeDrawer
                        isOpen={true}
                        onClose={vi.fn()}
                        currentShellId="1"
                        currentHostId="h-1"
                        currentHostName="web-01"
                        currentHostIp="10.0.0.10"
                        hosts={[...sampleHosts, fileserverHost]}
                        shells={[]}
                        pivots={[]}
                        availableCredentials={sampleCreds}
                        onHostSaved={onHostSaved}
                        onCredentialCreated={onCredentialCreated}
                    />
                </BrowserRouter>
            </MockedProvider>
        );

        fireEvent.click(screen.getByTestId("hosts-tree-add-credential-button"));
        expect(screen.getByTestId("credential-modal")).toBeInTheDocument();

        fireEvent.change(screen.getByLabelText("Principal / Username"), {
            target: { value: "postgres" },
        });
        fireEvent.change(screen.getByLabelText("Secret / Password"), {
            target: { value: "PgPass2026!" },
        });
        fireEvent.change(screen.getByLabelText("Host"), {
            target: { value: "nh-30" },
        });

        fireEvent.click(screen.getByTestId("credential-modal-submit"));

        await waitFor(() => {
            expect(onCredentialCreated).toHaveBeenCalledWith(
                expect.objectContaining({ id: "cred-100", principal: "postgres" }),
                "nh-30"
            );
        });

        // 4. Verify editing discovered host db-01 via HostModal (ports + credential association)
        render(
            <MockedProvider mocks={[updateNetworkHostMock]} addTypename={false}>
                <HostModal
                    isOpen={true}
                    onClose={vi.fn()}
                    initialHost={sampleHosts[0]}
                    availableCredentials={sampleCreds}
                    onHostSaved={onHostSaved}
                />
            </MockedProvider>
        );

        expect(screen.getByTestId("host-modal")).toBeInTheDocument();
        expect(screen.getByText("Edit Host")).toBeInTheDocument();

        const port5432Pill = screen.getByTestId("host-port-pill-5432");
        expect(port5432Pill).toHaveAttribute("aria-pressed", "true");
        fireEvent.click(port5432Pill);
        expect(port5432Pill).toHaveAttribute("aria-pressed", "false");
        expect((screen.getByLabelText("Open Ports") as HTMLInputElement).value).toBe("22");

        fireEvent.change(screen.getByLabelText("Open Ports"), {
            target: { value: "22, 443" },
        });

        const credOption = screen.getByTestId("host-cred-option-cred-1");
        expect(credOption).toHaveAttribute("aria-pressed", "false");
        fireEvent.click(credOption);
        expect(credOption).toHaveAttribute("aria-pressed", "true");

        fireEvent.click(screen.getByTestId("host-modal-submit"));

        await waitFor(() => {
            expect(onHostSaved).toHaveBeenCalledWith(
                expect.objectContaining({
                    id: "h-2",
                    networkHostId: "nh-2",
                    openPorts: [22, 443],
                }),
                "edit"
            );
        });
    });

    it("automatically associates CredentialModal submissions with the current host resolved from hosts when currentHostId prop is omitted", async () => {
        const onCredentialCreated = vi.fn();
        const fallbackCredMock = {
            request: {
                query: CREATE_CREDENTIAL_MUTATION,
                variables: {
                    input: {
                        hostID: "h-curr-fallback",
                        principal: "svc-auto",
                        secret: "AutoAssoc2026!",
                        kind: "KIND_PASSWORD",
                    },
                },
            },
            result: {
                data: {
                    createCredential: {
                        id: "cred-fallback-1",
                        principal: "svc-auto",
                        kind: "KIND_PASSWORD",
                        secret: "AutoAssoc2026!",
                        createdAt: "2026-01-01T00:00:00Z",
                    },
                },
            },
        };

        render(
            <MockedProvider mocks={[fallbackCredMock]} addTypename={false}>
                <CredentialModal
                    isOpen={true}
                    onClose={vi.fn()}
                    hosts={[
                        {
                            id: "h-curr-fallback",
                            hostId: "h-curr-fallback",
                            name: "pivot-box",
                            primaryIP: "10.0.5.5",
                            tags: [],
                            beaconCount: 1,
                            credentials: [],
                            credentialCount: 0,
                            isCurrentHost: true,
                            sharedTags: [],
                            sameSubnet: true,
                        },
                    ]}
                    onCredentialCreated={onCredentialCreated}
                />
            </MockedProvider>
        );

        expect(screen.getByLabelText("Host")).toHaveTextContent("pivot-box (10.0.5.5)");
        fireEvent.change(screen.getByLabelText("Principal / Username"), {
            target: { value: "svc-auto" },
        });
        fireEvent.change(screen.getByLabelText("Secret / Password"), {
            target: { value: "AutoAssoc2026!" },
        });
        fireEvent.click(screen.getByTestId("credential-modal-submit"));

        await waitFor(() => {
            expect(onCredentialCreated).toHaveBeenCalledWith(
                expect.objectContaining({
                    id: "cred-fallback-1",
                    principal: "svc-auto",
                    hostId: "h-curr-fallback",
                    isCurrentHost: true,
                }),
                undefined
            );
        });
    });

    it("prevents hidden TabPanels from corrupting PTY dimensions and preserves htop alternate-screen redraws", () => {
        const tabPanel = document.createElement("div");
        const termDiv = document.createElement("div");
        termDiv.style.height = "100%";
        termDiv.style.width = "100%";
        tabPanel.appendChild(termDiv);
        document.body.appendChild(tabPanel);

        try {
            // When TabPanel has `hidden` attribute (inactive tab), isTerminalElementVisible must return false
            // so xterm-addon-fit never parses `height: "100%"` as 100px (4 rows x 9 cols)
            tabPanel.setAttribute("hidden", "");
            expect(isTerminalElementVisible(termDiv)).toBe(false);

            // When TabPanel is active and has layout dimensions, isTerminalElementVisible returns true
            tabPanel.removeAttribute("hidden");
            Object.defineProperty(termDiv, "clientWidth", { value: 1280, configurable: true });
            Object.defineProperty(termDiv, "clientHeight", { value: 720, configurable: true });
            expect(isTerminalElementVisible(termDiv)).toBe(true);
        } finally {
            document.body.removeChild(tabPanel);
        }

        // Full-screen TUIs (htop, vim, less) in alternate screen buffer must not have \x1b[2J converted into newline scrolls
        const htopRedraw = "\x1b[H\x1b[2J\x1b[2d  1  [||||12.5%]\x1b[24;1HF1Help  F10Quit";
        expect(transformClearAnsiForScrollback(htopRedraw, 24, 24, true)).toBe(htopRedraw);

        const htopEnterAlt = "\x1b[?1049h\x1b[H\x1b[2J  1  [||||12.5%]";
        expect(transformClearAnsiForScrollback(htopEnterAlt, 24, 10, false)).toBe(htopEnterAlt);

        // Normal buffer `clear` (\x1b[H\x1b[2J\x1b[3J) strips \x1b[3J and scrolls non-empty lines into scrollback
        const normalClear = "\x1b[H\x1b[2J\x1b[3J";
        expect(transformClearAnsiForScrollback(normalClear, 24, 3, false)).toBe(
            "\x1b[24;1H\n\n\n\x1b[H\x1b[J"
        );
    });
});

describe("ShellScratchpad", () => {
    it("loads existing OperatorNote across shells and adds shellId only when not already present on modification", async () => {
        localStorage.removeItem(SCRATCHPAD_STORAGE_KEY);
        const onToggle = vi.fn();
        let updateAddedShellOnce = false;
        let updateWithoutShellOnce = false;

        const mocks = [
            {
                request: {
                    query: GET_OPERATOR_NOTES,
                },
                result: {
                    data: {
                        operatorNotes: {
                            edges: [
                                {
                                    node: {
                                        id: "note-1",
                                        content: "Initial shared note",
                                        createdAt: "2025-01-01T00:00:00Z",
                                        lastModifiedAt: "2025-01-01T00:00:00Z",
                                        author: {
                                            id: "u-1",
                                            name: "op",
                                        },
                                        shells: {
                                            edges: [
                                                {
                                                    node: {
                                                        id: "shell-1",
                                                    },
                                                },
                                            ],
                                        },
                                    },
                                },
                            ],
                        },
                    },
                },
            },
            {
                request: {
                    query: UPDATE_OPERATOR_NOTE_MUTATION,
                    variables: {
                        operatorNoteID: "note-1",
                        input: {
                            content: "Updated from shell-2",
                            addShellIDs: ["shell-2"],
                        },
                    },
                },
                result: () => {
                    updateAddedShellOnce = true;
                    return {
                        data: {
                            updateOperatorNote: {
                                id: "note-1",
                                content: "Updated from shell-2",
                                createdAt: "2025-01-01T00:00:00Z",
                                lastModifiedAt: "2025-01-01T00:01:00Z",
                                author: {
                                    id: "u-1",
                                    name: "op",
                                },
                                shells: {
                                    edges: [
                                        { node: { id: "shell-1" } },
                                        { node: { id: "shell-2" } },
                                    ],
                                },
                            },
                        },
                    };
                },
            },
            {
                request: {
                    query: UPDATE_OPERATOR_NOTE_MUTATION,
                    variables: {
                        operatorNoteID: "note-1",
                        input: {
                            content: "Updated again in shell-2",
                        },
                    },
                },
                result: () => {
                    updateWithoutShellOnce = true;
                    return {
                        data: {
                            updateOperatorNote: {
                                id: "note-1",
                                content: "Updated again in shell-2",
                                createdAt: "2025-01-01T00:00:00Z",
                                lastModifiedAt: "2025-01-01T00:02:00Z",
                                author: {
                                    id: "u-1",
                                    name: "op",
                                },
                                shells: {
                                    edges: [
                                        { node: { id: "shell-1" } },
                                        { node: { id: "shell-2" } },
                                    ],
                                },
                            },
                        },
                    };
                },
            },
        ];

        render(
            <MockedProvider mocks={mocks} addTypename={false}>
                <ShellScratchpad
                    isOpen={true}
                    onToggle={onToggle}
                    shellId="shell-2"
                    fontSize={13}
                />
            </MockedProvider>
        );

        const textarea = screen.getByTestId("scratchpad-textarea") as HTMLTextAreaElement;
        await waitFor(() => {
            expect(textarea.value).toBe("Initial shared note");
        });

        // Modify the note in shell-2 (which is not yet in shells) -> should send addShellIDs: ["shell-2"]
        fireEvent.change(textarea, { target: { value: "Updated from shell-2" } });
        fireEvent.blur(textarea);

        await waitFor(() => {
            expect(updateAddedShellOnce).toBe(true);
        });

        // Modify the note again in shell-2 -> shell-2 is already in shells, so addShellIDs is omitted
        fireEvent.change(textarea, { target: { value: "Updated again in shell-2" } });
        fireEvent.blur(textarea);

        await waitFor(() => {
            expect(updateWithoutShellOnce).toBe(true);
        });
    });

    it("supports offline mode when loading fails and syncs the note the next time the backend is available", async () => {
        localStorage.removeItem(SCRATCHPAD_STORAGE_KEY);
        let createCalled = false;

        const offlineMocks = [
            {
                request: {
                    query: GET_OPERATOR_NOTES,
                },
                error: new Error("Network unavailable"),
            },
        ];

        const { unmount } = render(
            <MockedProvider mocks={offlineMocks} addTypename={false}>
                <ShellScratchpad
                    isOpen={true}
                    onToggle={vi.fn()}
                    shellId="shell-offline-1"
                />
            </MockedProvider>
        );

        // Wait for offline badge to appear after query failure
        await waitFor(() => {
            expect(screen.getByTestId("scratchpad-offline-badge")).toBeInTheDocument();
        });

        const textarea = screen.getByTestId("scratchpad-textarea") as HTMLTextAreaElement;
        fireEvent.change(textarea, {
            target: { value: "Saved while offline in shell-offline-1" },
        });

        const savedOffline = loadScratchpadState();
        expect(savedOffline.content).toBe("Saved while offline in shell-offline-1");
        expect(savedOffline.pendingSync).toBe(true);
        expect(savedOffline.pendingShellIds).toEqual(["shell-offline-1"]);

        unmount();

        // Next time the scratchpad opens when the backend is available, it automatically syncs the offline note
        const onlineMocks = [
            {
                request: {
                    query: GET_OPERATOR_NOTES,
                },
                result: {
                    data: {
                        operatorNotes: {
                            edges: [],
                        },
                    },
                },
            },
            {
                request: {
                    query: CREATE_OPERATOR_NOTE_MUTATION,
                    variables: {
                        input: {
                            content: "Saved while offline in shell-offline-1",
                            shellIDs: ["shell-offline-1"],
                        },
                    },
                },
                result: () => {
                    createCalled = true;
                    return {
                        data: {
                            createOperatorNote: {
                                id: "note-99",
                                content: "Saved while offline in shell-offline-1",
                                createdAt: "2025-01-01T00:00:00Z",
                                lastModifiedAt: "2025-01-01T00:00:00Z",
                                author: {
                                    id: "u-1",
                                    name: "op",
                                },
                                shells: {
                                    edges: [{ node: { id: "shell-offline-1" } }],
                                },
                            },
                        },
                    };
                },
            },
        ];

        render(
            <MockedProvider mocks={onlineMocks} addTypename={false}>
                <ShellScratchpad
                    isOpen={true}
                    onToggle={vi.fn()}
                    shellId="shell-offline-2"
                />
            </MockedProvider>
        );

        const syncedTextarea = screen.getByTestId("scratchpad-textarea") as HTMLTextAreaElement;
        expect(syncedTextarea.value).toBe("Saved while offline in shell-offline-1");

        await waitFor(() => {
            expect(createCalled).toBe(true);
        });

        await waitFor(() => {
            expect(loadScratchpadState().pendingSync).toBe(false);
            expect(loadScratchpadState().noteId).toBe("note-99");
        });
        expect(screen.queryByTestId("scratchpad-offline-badge")).not.toBeInTheDocument();
    });

    it("recovers from stale noteId and stale pendingShellIds in localStorage after a server reset without showing Offline", async () => {
        localStorage.setItem(
            SCRATCHPAD_STORAGE_KEY,
            JSON.stringify({
                content: "Note from previous server session",
                noteId: "stale-note-id-123",
                shellIds: ["stale-shell-1"],
                pendingShellIds: ["stale-shell-2"],
                pendingSync: true,
            })
        );

        let fallbackCreateWithoutShellIdsCalled = false;

        const mocks = [
            {
                request: {
                    query: GET_OPERATOR_NOTES,
                },
                result: {
                    data: {
                        operatorNotes: {
                            edges: [],
                        },
                    },
                },
            },
            {
                request: {
                    query: CREATE_OPERATOR_NOTE_MUTATION,
                    variables: {
                        input: {
                            content: "Note from previous server session",
                            shellIDs: ["stale-shell-2"],
                        },
                    },
                },
                error: new Error("FOREIGN KEY constraint failed"),
            },
            {
                request: {
                    query: CREATE_OPERATOR_NOTE_MUTATION,
                    variables: {
                        input: {
                            content: "Note from previous server session",
                        },
                    },
                },
                result: () => {
                    fallbackCreateWithoutShellIdsCalled = true;
                    return {
                        data: {
                            createOperatorNote: {
                                id: "fresh-note-100",
                                content: "Note from previous server session",
                                createdAt: "2025-01-01T00:00:00Z",
                                lastModifiedAt: "2025-01-01T00:00:00Z",
                                author: {
                                    id: "u-1",
                                    name: "op",
                                },
                                shells: {
                                    edges: [],
                                },
                            },
                        },
                    };
                },
            },
        ];

        render(
            <MockedProvider mocks={mocks} addTypename={false}>
                <ShellScratchpad
                    isOpen={true}
                    onToggle={vi.fn()}
                    shellId="active-shell-10"
                />
            </MockedProvider>
        );

        await waitFor(() => {
            expect(fallbackCreateWithoutShellIdsCalled).toBe(true);
        });

        await waitFor(() => {
            expect(loadScratchpadState().pendingSync).toBe(false);
            expect(loadScratchpadState().noteId).toBe("fresh-note-100");
        });
        expect(screen.queryByTestId("scratchpad-offline-badge")).not.toBeInTheDocument();
    });

    it("renders synchronized line numbers in Scratchpad and shares vertical drawer height across Composer, Settings, and Scratchpad", async () => {
        resetSharedDrawerHeightForTests();
        localStorage.removeItem(SCRATCHPAD_STORAGE_KEY);

        const emptyNotesMock = {
            request: { query: GET_OPERATOR_NOTES },
            result: { data: { operatorNotes: { edges: [] } } },
        };

        const { unmount } = render(
            <MockedProvider mocks={[emptyNotesMock]} addTypename={false}>
                <ShellScratchpad isOpen={true} onToggle={vi.fn()} shellId="s-height-1" />
            </MockedProvider>
        );

        const textarea = screen.getByTestId("scratchpad-textarea") as HTMLTextAreaElement;
        fireEvent.change(textarea, { target: { value: "line 1\nline 2\nline 3" } });

        const gutter = screen.getByTestId("scratchpad-line-numbers");
        expect(gutter).toHaveTextContent("1");
        expect(gutter).toHaveTextContent("2");
        expect(gutter).toHaveTextContent("3");

        // Resize Scratchpad via keyboard ArrowUp (+20px from default height)
        const resizeHandle = screen.getByTestId("scratchpad-resize-handle");
        fireEvent.keyDown(resizeHandle, { key: "ArrowUp" });

        const storedHeight = localStorage.getItem("tavern_shell_drawer_height_v1");
        expect(storedHeight).not.toBeNull();
        unmount();

        // Composer reads and uses the exact shared height
        render(
            <MultiLineComposer
                isOpen={true}
                onToggle={vi.fn()}
                code="print(1)"
                onChange={vi.fn()}
                onRunCode={vi.fn()}
            />
        );
        const composerEditor = screen.getByLabelText("Multi-line Eldritch script editor");
        const composerHeightWrapper = composerEditor.closest('[style*="height"]');
        expect(composerHeightWrapper).toHaveStyle({ height: `${storedHeight}px` });
    });
});

describe("Knowledge Graph deduplication, pivot filtering & beacon liveness", () => {
    it("deduplicates Host and NetworkHost sharing the same primary IP, hides failed closed pivots, and sorts live shells before dead ones", () => {
        const now = Date.now();
        const futureIso = new Date(now + 60_000).toISOString();
        const staleIso = new Date(now - 600_000).toISOString();

        expect(isBeaconOnline({ nextSeenAt: futureIso, interval: 60 }, now)).toBe(true);
        expect(isBeaconOnline({ nextSeenAt: staleIso, interval: 60 }, now)).toBe(false);

        expect(hasPivotActivity({ closedAt: null, data: "" })).toBe(true);
        expect(hasPivotActivity({ closedAt: "2025-01-01T00:00:00Z", data: "" })).toBe(false);
        expect(
            hasPivotActivity({
                closedAt: "2025-01-01T00:00:00Z",
                data: "\x1b[31m[-] dial tcp 10.0.0.5:22: connection refused\x1b[0m\r\n",
            })
        ).toBe(false);
        expect(
            hasPivotActivity({
                closedAt: "2025-01-01T00:00:00Z",
                data: "root@app-01:~# whoami\nroot\n",
            })
        ).toBe(true);

        const res = buildUnifiedGraphData({
            currentHostNode: { id: "h-1", name: "jump", primaryIP: "10.0.0.1" },
            hostId: "h-1",
            currentShellId: "s-1",
            currentHostTags: [],
            currentShellPivots: [
                {
                    id: "piv-failed",
                    kind: "SSH",
                    destination: "root@10.0.0.50:22",
                    closedAt: "2025-01-01T00:00:00Z",
                    data: "[-] permission denied\n",
                },
                {
                    id: "piv-ok",
                    kind: "SSH",
                    destination: "root@10.0.0.50:22",
                    closedAt: null,
                    data: "",
                },
            ],
            knowledgeData: {
                hosts: {
                    edges: [
                        {
                            node: {
                                id: "h-50",
                                name: "app-01",
                                primaryIP: "10.0.0.50",
                                beacons: {
                                    edges: [
                                        {
                                            node: {
                                                id: "b-dead",
                                                name: "beacon-dead",
                                                principal: "root",
                                                nextSeenAt: staleIso,
                                                interval: 60,
                                                shells: {
                                                    edges: [
                                                        {
                                                            node: {
                                                                id: "s-dead",
                                                                createdAt: "2025-01-01T12:00:00Z",
                                                                closedAt: null,
                                                                activeUsers: { edges: [] },
                                                            },
                                                        },
                                                    ],
                                                },
                                            },
                                        },
                                        {
                                            node: {
                                                id: "b-live",
                                                name: "beacon-live",
                                                principal: "www-data",
                                                nextSeenAt: futureIso,
                                                interval: 60,
                                                shells: {
                                                    edges: [
                                                        {
                                                            node: {
                                                                id: "s-live",
                                                                createdAt: "2025-01-01T11:00:00Z",
                                                                closedAt: null,
                                                                activeUsers: {
                                                                    edges: [
                                                                        {
                                                                            node: {
                                                                                id: "u1",
                                                                                name: "alice",
                                                                                photoURL: "",
                                                                            },
                                                                        },
                                                                    ],
                                                                },
                                                            },
                                                        },
                                                    ],
                                                },
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                    ],
                },
                networkHosts: {
                    edges: [
                        {
                            node: {
                                id: "nh-50",
                                name: "app-01",
                                hostname: "app-01.corp",
                                ipAddress: "10.0.0.50",
                                openPorts: [22, 80],
                                host: null, // No explicit foreign key, but matches h-50 by IP & name
                                networks: { edges: [] },
                            },
                        },
                    ],
                },
            },
        });

        // Hosts deduplicated to 1 merged entry instead of 2 duplicates
        expect(res.networkHosts).toHaveLength(1);
        expect(res.networkHosts[0].openPorts).toEqual([22, 80]);
        expect(res.networkHosts[0].hostId).toBe("h-50");

        // Failed pivot is hidden
        expect(res.nearbyPivots.map((p) => p.id)).toEqual(["piv-ok"]);

        // Live shell sorts ahead of dead shell
        expect(res.hostShells.map((s) => s.id)).toEqual(["s-live", "s-dead"]);
        expect(res.hostShells[0].beaconOnline).toBe(true);
        expect(res.hostShells[0].activeUsers).toEqual([{ id: "u1", name: "alice", photoURL: "" }]);
        expect(res.hostShells[1].beaconOnline).toBe(false);
    });
});
