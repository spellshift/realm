import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { BrowserRouter } from "react-router-dom";
import { MockedProvider } from "@apollo/client/testing";
import "@testing-library/jest-dom";

import InterpreterStatusBanner from "../InterpreterStatusBanner";
import MultiLineComposer from "../MultiLineComposer";
import ShellHistoryDrawer from "../ShellHistoryDrawer";
import ShellKnowledgeDrawer from "../ShellKnowledgeDrawer";
import ShellNuxDrawer from "../ShellNuxDrawer";
import ShellWalkthrough from "../ShellWalkthrough";
import SshConnectionModal from "../SshConnectionModal";
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

    it("renders queued task status without stuck warning when waiting on beacon check-in", () => {
        const queuedTask: ShellTaskItem = {
            id: "t1",
            createdAt: new Date().toISOString(),
            input: 'print("queued cmd")',
        };

        render(<InterpreterStatusBanner {...baseProps} queuedTasks={[queuedTask]} />);

        expect(screen.getByTestId("queued-tasks-strip")).toBeInTheDocument();
        expect(screen.getByText(/1 command queued in sequence/i)).toBeInTheDocument();
        expect(screen.queryByTestId("stuck-interpreter-banner")).not.toBeInTheDocument();
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

describe("MultiLineComposer (Composer)", () => {
    it("allows editing multi-line scripts, inserting aliases snippet, and executing via Ctrl+Enter", () => {
        const onRunCode = vi.fn();
        const onStageToPrompt = vi.fn();
        const onChange = vi.fn();

        render(
            <MultiLineComposer
                isOpen={true}
                onToggle={vi.fn()}
                code={'for p in process.list():\n    print(p["name"])'}
                onChange={onChange}
                onRunCode={onRunCode}
                onStageToPrompt={onStageToPrompt}
            />
        );

        expect(screen.getByText("Composer")).toBeInTheDocument();
        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(textarea.value).toContain("process.list()");

        fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });
        expect(onRunCode).toHaveBeenCalledWith('for p in process.list():\n    print(p["name"])');

        // Selecting the aliases snippet (index 0) registers ls(), whoami(), ps(), netstat()
        const snippetSelect = screen.getByLabelText("Insert Eldritch snippet");
        fireEvent.change(snippetSelect, { target: { value: "0" } });
        expect(onChange).toHaveBeenCalled();
        const insertedCode = onChange.mock.calls[0][0] as string;
        expect(insertedCode).toContain("def ls(");
        expect(insertedCode).toContain("def whoami(");
        expect(insertedCode).toContain("def ps(");
        expect(insertedCode).toContain("def netstat(");
    });
});

describe("ShellHistoryDrawer", () => {
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

    it("displays task history, Knowledge Graph reported badges, output inspection, and closed pivot transcripts", () => {
        const onRunCode = vi.fn();
        const onLoadIntoComposer = vi.fn();
        const onOpenPivotTab = vi.fn();

        render(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={sampleTasks}
                localHistory={[]}
                pivots={samplePivots}
                onRunCode={onRunCode}
                onLoadIntoComposer={onLoadIntoComposer}
                onOpenPivotTab={onOpenPivotTab}
            />
        );

        expect(screen.getByText(/1 Cred Reported/i)).toBeInTheDocument();

        // View output
        fireEvent.click(screen.getByText("View Output"));
        expect(screen.getByText("Reported credential")).toBeInTheDocument();

        // Re-run command
        fireEvent.click(screen.getByText("Re-run"));
        expect(onRunCode).toHaveBeenCalledWith('report.user_password("admin", "hunter2")');

        // Switch to Pivot Sessions tab and open closed transcript
        fireEvent.click(screen.getByText(/Pivot Sessions/i));
        expect(screen.getByText(/SSH • root@10.0.0.15:22/i)).toBeInTheDocument();
        fireEvent.click(screen.getByText("View Saved Transcript"));
        expect(onOpenPivotTab).toHaveBeenCalledWith("ssh", "root@10.0.0.15:22", "42");
    });

    it("strictly filters Queued / Running commands without falling back to completed local history", () => {
        const activeTask: ShellTaskItem = {
            id: "task-queued",
            createdAt: "2025-01-01T12:05:00Z",
            input: 'sys.shell("sleep 10")',
        };

        render(
            <ShellHistoryDrawer
                isOpen={true}
                onClose={vi.fn()}
                shellTasks={[...sampleTasks, activeTask]}
                localHistory={['print("old local command")']}
                pivots={[]}
                onRunCode={vi.fn()}
                onLoadIntoComposer={vi.fn()}
                onOpenPivotTab={vi.fn()}
            />
        );

        // Both tasks visible under All
        expect(screen.getByText('report.user_password("admin", "hunter2")')).toBeInTheDocument();
        expect(screen.getByText('sys.shell("sleep 10")')).toBeInTheDocument();

        // Click "Queued / Running" filter -> only activeTask is shown
        fireEvent.click(screen.getByText("Queued / Running"));
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
            hash: "abc123",
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
