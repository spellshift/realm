import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import "@testing-library/jest-dom";

import InterpreterStatusBanner from "../InterpreterStatusBanner";
import MultiLineComposer from "../MultiLineComposer";
import ShellHistoryDrawer from "../ShellHistoryDrawer";
import ShellPivotsDrawer from "../ShellPivotsDrawer";
import ShellKnowledgeDrawer from "../ShellKnowledgeDrawer";
import ShellNuxDrawer from "../ShellNuxDrawer";
import ShellWalkthrough from "../ShellWalkthrough";
import SshConnectionModal from "../SshConnectionModal";
import { analyzeEldritchCode, completeEldritchCode } from "../../utils/eldritchLsp";
import { normalizeMultilineScript } from "../../../../lib/browser-adapter";
import {
    KnowledgeCredentialItem,
    KnowledgeFileItem,
    KnowledgeHostItem,
    ShellPivotItem,
    ShellTaskItem,
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
        onOpenHistoryDrawer: vi.fn(),
    };

    it("renders 1-time interactive walkthrough popover with component steps and dismisses cleanly", () => {
        const onDismiss = vi.fn();
        const onOpenGuideDrawer = vi.fn();

        render(
            <ShellWalkthrough
                isOpen={true}
                onDismiss={onDismiss}
                onOpenGuideDrawer={onOpenGuideDrawer}
            />
        );

        expect(screen.getByTestId("shell-nux-banner")).toBeInTheDocument();
        expect(screen.getByText(/Eldritch REPL & Multi-Line Editing/i)).toBeInTheDocument();

        // Advance through steps
        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText(/Interactive Portals & Pivots/i)).toBeInTheDocument();

        fireEvent.click(screen.getByText("Next"));
        expect(screen.getByText(/Composer & Shell Aliases/i)).toBeInTheDocument();

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

        rerender(
            <InterpreterStatusBanner
                {...baseProps}
                connectionStatus="disconnected"
                onReconnect={onReconnect}
            />
        );
        const reconnectBtn = screen.getByText(/Reconnect Now/i);
        expect(reconnectBtn).toBeInTheDocument();
        fireEvent.click(reconnectBtn);
        expect(onReconnect).toHaveBeenCalled();
    });

    it("renders stuck interpreter warning and triggers Spawn new shell", () => {
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

        fireEvent.click(screen.getByText(/Spawn new shell/i));
        expect(onSpawnFreshShell).toHaveBeenCalled();
    });
});

describe("MultiLineComposer (Composer) & eldritch-lsp", () => {
    it("allows editing multi-line scripts, inserting valid aliases snippet, syntax highlighting, and executing via Ctrl+Enter", () => {
        const onRunCode = vi.fn();
        const onChange = vi.fn();

        render(
            <MultiLineComposer
                isOpen={true}
                onToggle={vi.fn()}
                code={'for p in process.list():\n    print(p["name"])'}
                onChange={onChange}
                onRunCode={onRunCode}
            />
        );

        expect(screen.getByText("Composer")).toBeInTheDocument();
        expect(screen.getByTestId("composer-syntax-highlight")).toBeInTheDocument();
        expect(screen.getByTestId("eldritch-lsp-status")).toHaveTextContent(/eldritch-lsp/i);
        expect(screen.getByTestId("composer-resize-handle")).toBeInTheDocument();

        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(textarea.value).toContain("process.list()");

        fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });
        expect(onRunCode).toHaveBeenCalledWith('for p in process.list():\n    print(p["name"])');

        // Selecting the aliases snippet (index 0) registers ls(), pwd(), cat(), whoami(), ps(), netstat() with 0 LSP diagnostics
        const snippetSelect = screen.getByLabelText("Insert Eldritch snippet");
        fireEvent.change(snippetSelect, { target: { value: "0" } });
        expect(onChange).toHaveBeenCalled();
        const insertedCode = onChange.mock.calls[0][0] as string;
        expect(insertedCode).toContain("def ls(");
        expect(insertedCode).toContain("def whoami(");
        expect(insertedCode).toContain("def ps(");
        expect(insertedCode).toContain("def netstat(");
        expect(insertedCode).not.toContain("file.moveto");

        const diagnostics = analyzeEldritchCode(normalizeMultilineScript(insertedCode));
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

        // Should flag deprecated sys.get_os() via eldritch-lsp and offer 1-click replacement
        expect(screen.getByTestId("composer-diagnostics-list")).toBeInTheDocument();
        const quickFixBtn = screen.getByText("Replace with sys.info");
        fireEvent.click(quickFixBtn);

        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(textarea.value).toBe("os_info = sys.info()");

        // Undo via Ctrl+Z (Linux/Windows) or Cmd+Z (macOS)
        fireEvent.keyDown(textarea, { key: "z", metaKey: true });
        expect(textarea.value).toBe("os_info = sys.get_os()");

        // Redo via Ctrl+Shift+Z / Cmd+Shift+Z
        fireEvent.keyDown(textarea, { key: "z", metaKey: true, shiftKey: true });
        expect(textarea.value).toBe("os_info = sys.info()");
    });

    it("provides context-aware autocomplete (Tab / Left / Right cycle, Enter accept) and indentation linting via eldritch-lsp", () => {
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
        expect(screen.getByTestId("composer-autocomplete-list")).toBeInTheDocument();

        const firstOption = completions!.items[0];
        const secondOption = completions!.items[1];

        // Tab cycles to second option
        fireEvent.keyDown(textarea, { key: "Tab" });
        // ArrowRight cycles to third option, ArrowLeft cycles back to second option
        fireEvent.keyDown(textarea, { key: "ArrowRight" });
        fireEvent.keyDown(textarea, { key: "ArrowLeft" });
        // Enter accepts the selected (second) option
        fireEvent.keyDown(textarea, { key: "Enter" });

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

    it("displays History drawer titled 'History' and renders pivots in dedicated ShellPivotsDrawer", () => {
        const onRunCode = vi.fn();
        const onLoadIntoComposer = vi.fn();
        const onOpenPivotTab = vi.fn();

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
                    onOpenPty={vi.fn()}
                    onOpenSshModal={vi.fn()}
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

        // Dedicated Pivots drawer displays pivot sessions and opens saved transcript
        expect(screen.getByTestId("shell-pivots-drawer")).toBeInTheDocument();
        expect(screen.getByText(/SSH • root@10.0.0.15:22/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText("View Saved Transcript"));
        expect(onOpenPivotTab).toHaveBeenCalledWith("ssh", "root@10.0.0.15:22", "42");
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

        // Increment focusQueueTrigger (simulating clicking the bottom status bar spinner) -> switches to Queued / Running
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
    it("renders Quick Start Guide and searchable Eldritch v0.4.1 Stdlib reference", () => {
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

        expect(screen.getByText(/1. Queued Mode vs. Interactive Portal Mode/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText(/Run portal.create\(\)/i));
        expect(onRunCode).toHaveBeenCalledWith("portal.create()");

        // Switch to Eldritch Stdlib tab
        fireEvent.click(screen.getByText(/Eldritch v0.4.1 Stdlib/i));
        const searchInput = screen.getByPlaceholderText(/Search Eldritch functions/i);
        fireEvent.change(searchInput, { target: { value: "file.list" } });
        expect(screen.getByText("file.list")).toBeInTheDocument();
    });
});
