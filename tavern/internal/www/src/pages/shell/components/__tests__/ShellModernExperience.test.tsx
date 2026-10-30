import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import "@testing-library/jest-dom";

import InterpreterStatusBanner from "../InterpreterStatusBanner";
import MultiLineComposer, {
    DEFAULT_RC_ALIASES_SCRIPT,
    loadRcEnabled,
    loadRcScript,
} from "../MultiLineComposer";
import SessionSettings from "../SessionSettings";
import ShellHistoryDrawer from "../ShellHistoryDrawer";
import ShellPivotsDrawer from "../ShellPivotsDrawer";
import ShellKnowledgeDrawer from "../ShellKnowledgeDrawer";
import ShellNuxDrawer from "../ShellNuxDrawer";
import ShellTerminal from "../ShellTerminal";
import ShellWalkthrough from "../ShellWalkthrough";
import SshConnectionModal from "../SshConnectionModal";
import { formatPivotTabLabel } from "../../index";
import { analyzeEldritchCode, completeEldritchCode } from "../../utils/eldritchLsp";
import {
    BrowserWasmAdapter,
    WEBSOCKET_CONNECT_TIMEOUT_MS,
    getExponentialBackoffDelay,
    normalizeMultilineScript,
} from "../../../../lib/browser-adapter";
import {
    KnowledgeCredentialItem,
    KnowledgeFileItem,
    KnowledgeHostItem,
    ShellPivotItem,
    ShellTaskItem,
    ShellTomeItem,
} from "../../hooks/useShellData";

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

        // Step 3: Setup Aliases (opens SessionSettings drawer and highlights Setup Aliases button)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Setup Aliases")).toBeInTheDocument();
        expect(onOpenSessionSettings).toHaveBeenCalledTimes(1);

        // Step 4: Async Mode (positioned above bottom status bar element without overlapping)
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
                toJSON: () => {},
            } as DOMRect);
        document.body.appendChild(nonInteractiveEl);

        fireEvent.click(screen.getByText("Next"));
        const dialog = screen.getByRole("dialog", { name: "Async Mode" });
        expect(dialog).toBeInTheDocument();
        expect(dialog.style.bottom).toBe(`${window.innerHeight - 760 + 14}px`);
        expect(dialog.style.top).toBe("");
        document.body.removeChild(nonInteractiveEl);

        // Step 5: Agent Callback
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Agent Callback")).toBeInTheDocument();

        // Step 6: Queued & Executing Tasks (after Agent Callback)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Queued & Executing Tasks")).toBeInTheDocument();
        expect(onQueueStepChange).toHaveBeenLastCalledWith(true);

        // Step 7: Eldritch Composer (after Queued Tasks, before Knowledge Graph; opens composer)
        onCloseComposer.mockClear();
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Eldritch Composer")).toBeInTheDocument();
        expect(onOpenComposer).toHaveBeenCalledTimes(1);
        expect(onCloseComposer).not.toHaveBeenCalled();
        expect(onQueueStepChange).toHaveBeenLastCalledWith(false);

        // Step 8: Knowledge Graph (closes composer when moving on to the next NUX)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Knowledge Graph")).toBeInTheDocument();
        expect(onCloseComposer).toHaveBeenCalledTimes(1);

        // Step 9: Pivots
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Pivots")).toBeInTheDocument();

        // Step 10: Command Queue & History
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Command Queue & History")).toBeInTheDocument();

        // Step 11: Operator Guide & Stdlib Reference
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Operator Guide & Stdlib Reference")).toBeInTheDocument();

        // Step 12: Going Interactive (opens New Portal menu option)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Going Interactive")).toBeInTheDocument();
        expect(onPortalStepChange).toHaveBeenLastCalledWith(true);

        // Step 13: Feedback & Bug Reports (last two are portal and report a bug)
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText("Feedback & Bug Reports")).toBeInTheDocument();
        expect(onPortalStepChange).toHaveBeenLastCalledWith(true);
        expect(screen.getByText(/Thank you for helping make Realm better!/i)).toBeInTheDocument();

        fireEvent.click(screen.getByLabelText("Dismiss welcome guide"));
        expect(onDismiss).toHaveBeenCalled();
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
        render(
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
        expect(screen.getByText(/Interpreter Busy or Potentially Stuck/i)).toBeInTheDocument();

        fireEvent.click(screen.getByText(/New Interpreter/i));
        expect(onSpawnFreshShell).toHaveBeenCalled();
    });

    it("enforces a 10s WebSocket connection timeout and retries with exponential backoff", () => {
        expect(WEBSOCKET_CONNECT_TIMEOUT_MS).toBe(10_000);
        expect(getExponentialBackoffDelay(0)).toBe(1_000);
        expect(getExponentialBackoffDelay(1)).toBe(2_000);
        expect(getExponentialBackoffDelay(2)).toBe(4_000);
        expect(getExponentialBackoffDelay(3)).toBe(8_000);
        expect(getExponentialBackoffDelay(4)).toBe(16_000);
        expect(getExponentialBackoffDelay(5)).toBe(30_000);

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

            // Initial socket created; does not time out at 5s
            expect(createdSockets).toHaveLength(1);
            vi.advanceTimersByTime(5_000);
            expect(createdSockets).toHaveLength(1);
            expect(statuses).toEqual([]);

            // At 10s, initial connection attempt times out and schedules retry #1 (1s backoff)
            vi.advanceTimersByTime(5_000);
            expect(statuses).toContain("reconnecting");
            expect(createdSockets).toHaveLength(1);

            vi.advanceTimersByTime(1_000);
            expect(createdSockets).toHaveLength(2);

            // Second attempt also hangs for 10s -> schedules retry #2 (2s backoff)
            vi.advanceTimersByTime(10_000);
            expect(createdSockets).toHaveLength(2);
            vi.advanceTimersByTime(1_999);
            expect(createdSockets).toHaveLength(2);
            vi.advanceTimersByTime(1);
            expect(createdSockets).toHaveLength(3);

            // Third attempt fails immediately via onclose -> schedules retry #3 (4s backoff)
            createdSockets[2].onclose?.({ reason: "network error" });
            vi.advanceTimersByTime(3_999);
            expect(createdSockets).toHaveLength(3);
            vi.advanceTimersByTime(1);
            expect(createdSockets).toHaveLength(4);

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
        expect(screen.getByTestId("eldritch-lsp-status")).toBeInTheDocument();
        expect(screen.getByTestId("eldritch-lsp-status")).not.toHaveTextContent("eldritch-lsp");
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

        // SessionSettings drawer renders radio toggle and initial chooser with "Setup Aliases" above "Load RC Tome" or "or custom"
        expect(screen.getByTestId("session-settings-drawer")).toBeInTheDocument();
        const rcToggle = screen.getByTestId("session-settings-rc-toggle");
        expect(rcToggle).toHaveAttribute("aria-checked", "false");
        expect(rcToggle).not.toHaveClass("bg-purple-600");

        expect(screen.getByTestId("session-settings-initial-chooser")).toBeInTheDocument();
        const setupAliasesBtn = screen.getByTestId("session-settings-shell-aliases-setup");
        expect(setupAliasesBtn).toHaveTextContent("Setup Aliases");
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

        // Lint checkmark matches composer view (green text-emerald-400)
        expect(screen.getByTestId("rc-eldritch-lsp-status")).toHaveClass("text-emerald-400");

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
        expect(screen.getByText(/1 Cred Reported/i)).toBeInTheDocument();

        // View output
        fireEvent.click(screen.getByText("View Output"));
        expect(screen.getByText("Reported credential")).toBeInTheDocument();

        // Re-run command
        fireEvent.click(screen.getByText("Re-run"));
        expect(onRunCode).toHaveBeenCalledWith('report.user_password("admin", "hunter2")');

        // Dedicated Pivots drawer displays PTY and SSH buttons and opens saved transcript
        expect(screen.getByTestId("shell-pivots-drawer")).toBeInTheDocument();
        fireEvent.click(screen.getByRole("button", { name: "PTY" }));
        expect(onOpenPty).toHaveBeenCalledTimes(1);
        fireEvent.click(screen.getByRole("button", { name: "SSH" }));
        expect(onOpenSshModal).toHaveBeenCalledTimes(1);
        expect(screen.getByText(/SSH • root@10.0.0.15:22/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText("View Saved Transcript"));
        expect(onOpenPivotTab).toHaveBeenCalledWith("ssh", "root@10.0.0.15:22", "42");

        // Pivot tab labels omit IDs and show IP for SSH or truncated last command for PTY (ignoring "exit")
        expect(formatPivotTabLabel({ type: "ssh", target: "root:secret@10.0.0.15:22" })).toBe("10.0.0.15");
        expect(formatPivotTabLabel({ type: "pty", target: "PTY #90194313217" })).toBe("PTY");
        expect(formatPivotTabLabel({ type: "pty", target: "exit" })).toBe("PTY");
        expect(
            formatPivotTabLabel({
                type: "pty",
                target: "cat /etc/shadow && grep root /etc/passwd",
            })
        ).toBe("cat /etc/shadow && grep ...");
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

        render(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={currentSessionTasks}
                allShellTasks={crossSessionTasks}
                currentShellId="10"
                onRunCode={vi.fn()}
                onLoadIntoComposer={vi.fn()}
            />
        );

        // Two identical whoami() runs are aggregated with a ×2 badge
        expect(screen.getByTestId("history-aggregated-count")).toHaveTextContent("×2");
        // Cross-session task from Shell #99 is visible by default under "All Sessions"
        expect(screen.getByText("Shell #99")).toBeInTheDocument();

        // Switch to Output tab on the long-output task and verify truncation + Show full toggle
        const viewOutputButtons = screen.getAllByText("View Output");
        fireEvent.click(viewOutputButtons[0]);
        expect(screen.queryByText(/line 5/)).not.toBeInTheDocument();

        const toggleFullBtn = screen.getByTestId("history-toggle-full");
        expect(toggleFullBtn).toHaveTextContent(/Show full/i);
        fireEvent.click(toggleFullBtn);
        expect(screen.getByText(/line 5/)).toBeInTheDocument();

        // Toggle scope to Current Session only -> Shell #99 task is hidden
        const scopeToggle = screen.getByTestId("history-session-scope-toggle");
        expect(scopeToggle).toHaveTextContent("All Sessions");
        fireEvent.click(scopeToggle);
        expect(scopeToggle).toHaveTextContent("Current Session");
        expect(screen.queryByText("Shell #99")).not.toBeInTheDocument();
    });
});

describe("ShellKnowledgeDrawer & SshConnectionModal", () => {
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
            name: "db-01",
            primaryIP: "10.0.0.20",
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

    const sampleFiles: KnowledgeFileItem[] = [
        {
            id: "file-77",
            path: "/etc/shadow.bak",
            size: 1024,
            permissions: "-rw-------",
            owner: "root",
            createdAt: "2025-01-01T12:00:00Z",
        },
    ];

    it("renders live Knowledge Graph drawer, stages credentials, and provides download links for reported files", () => {
        const onOpenSshWithTarget = vi.fn();
        const onLoadIntoComposer = vi.fn();

        render(
            <MockedProvider mocks={[]} addTypename={false}>
                <BrowserRouter>
                    <ShellKnowledgeDrawer
                        isOpen={true}
                        onClose={vi.fn()}
                        currentShellId="1"
                        currentHostId="h-1"
                        currentHostName="web-01"
                        currentHostIp="10.0.0.10"
                        portalId={99}
                        hostCredentials={sampleCreds}
                        allCredentials={sampleCreds}
                        networkHosts={sampleHosts}
                        hostFiles={sampleFiles}
                        hostProcessCount={42}
                        hostShells={[]}
                        lastKnowledgeSyncAt={Date.now()}
                        onRefetchKnowledgeGraph={vi.fn()}
                        onOpenSshWithTarget={onOpenSshWithTarget}
                        onLoadIntoComposer={onLoadIntoComposer}
                        onStageToPrompt={vi.fn()}
                    />
                </BrowserRouter>
            </MockedProvider>
        );

        expect(screen.getByText("Knowledge Graph")).toBeInTheDocument();
        expect(screen.getByText("svc_backup")).toBeInTheDocument();

        // Reveal secret
        fireEvent.click(screen.getByLabelText("Reveal secret"));
        expect(screen.getByText("Winter2025!")).toBeInTheDocument();

        // Stage ssh.exec in composer
        fireEvent.click(screen.getByText("ssh.exec"));
        expect(onLoadIntoComposer).toHaveBeenCalled();

        // Launch SSH Pivot
        fireEvent.click(screen.getByText("SSH Pivot"));
        expect(onOpenSshWithTarget).toHaveBeenCalledWith("svc_backup:Winter2025!@10.0.0.10:22");

        // Switch to Host Context tab and verify reported file download link
        fireEvent.click(screen.getByText(/Host Context/i));
        const downloadLink = screen.getByLabelText("Download /etc/shadow.bak") as HTMLAnchorElement;
        expect(downloadLink).toBeInTheDocument();
        expect(downloadLink.getAttribute("href")).toBe("/cdn/hostfiles/file-77");
        expect(downloadLink.hasAttribute("download")).toBe(true);
    });

    it("populates SshConnectionModal target from Knowledge Graph hosts and credentials", () => {
        const onConnect = vi.fn();
        render(
            <SshConnectionModal
                isOpen={true}
                onClose={vi.fn()}
                onConnect={onConnect}
                credentials={sampleCreds}
                hosts={sampleHosts}
            />
        );

        // Select host db-01 (10.0.0.20) and credential svc_backup
        fireEvent.click(screen.getByText("db-01"));
        fireEvent.click(screen.getByText("svc_backup"));

        const input = screen.getByLabelText("Target") as HTMLInputElement;
        expect(input.value).toBe("svc_backup:Winter2025!@10.0.0.20:22");

        fireEvent.click(screen.getByText("Connect"));
        expect(onConnect).toHaveBeenCalledWith("svc_backup:Winter2025!@10.0.0.20:22");
    });
});

describe("ShellNuxDrawer", () => {
    it("renders Quick Start Guide with top documentation link and searchable Eldritch v0.4.1 reference", () => {
        const onRunCode = vi.fn();
        const onStageToPrompt = vi.fn();

        render(
            <ShellNuxDrawer
                isOpen={true}
                onClose={vi.fn()}
                portalId={null}
                onRunCode={onRunCode}
                onStageToPrompt={onStageToPrompt}
                onLoadIntoComposer={vi.fn()}
                onSpawnFreshShell={vi.fn()}
            />
        );

        expect(screen.getByText("Read Full Documentation")).toBeInTheDocument();
        expect(screen.getByText(/Interactive Portals/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText(/Run portal.create\(\)/i));
        expect(onRunCode).toHaveBeenCalledWith("portal.create()");

        // Switch to Eldritch v0.4.1 tab
        fireEvent.click(screen.getByText("Eldritch v0.4.1"));
        const searchInput = screen.getByPlaceholderText(/Search Eldritch functions/i);
        fireEvent.change(searchInput, { target: { value: "file.list" } });
        expect(screen.getByText("file.list")).toBeInTheDocument();
    });
});
