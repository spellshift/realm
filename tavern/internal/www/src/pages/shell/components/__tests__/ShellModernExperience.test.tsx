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
import SshConnectionModal from "../SshConnectionModal";
import {
    KnowledgeCredentialItem,
    KnowledgeHostItem,
    ShellPivotItem,
    ShellTaskItem,
} from "../../hooks/useShellData";

describe("InterpreterStatusBanner", () => {
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
        onOpenNuxDrawer: vi.fn(),
        showNuxWelcome: false,
        onDismissNuxWelcome: vi.fn(),
        portalId: null as number | null,
        onOpenPortal: vi.fn(),
    };

    it("renders NUX welcome banner and triggers guide / portal / dismiss actions", () => {
        const onOpenNuxDrawer = vi.fn();
        const onOpenPortal = vi.fn();
        const onDismissNuxWelcome = vi.fn();

        render(
            <InterpreterStatusBanner
                {...baseProps}
                showNuxWelcome={true}
                onOpenNuxDrawer={onOpenNuxDrawer}
                onOpenPortal={onOpenPortal}
                onDismissNuxWelcome={onDismissNuxWelcome}
            />
        );

        expect(screen.getByTestId("shell-nux-banner")).toBeInTheDocument();
        fireEvent.click(screen.getByText(/Operator Guide & Docs/i));
        expect(onOpenNuxDrawer).toHaveBeenCalled();

        fireEvent.click(screen.getByText(/Open Interactive Portal/i));
        expect(onOpenPortal).toHaveBeenCalled();

        fireEvent.click(screen.getByLabelText("Dismiss welcome guide"));
        expect(onDismissNuxWelcome).toHaveBeenCalled();
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

    it("renders stuck interpreter warning and triggers Spawn Fresh Shell", () => {
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

        fireEvent.click(screen.getByText(/Spawn Fresh Shell \(New Interpreter\)/i));
        expect(onSpawnFreshShell).toHaveBeenCalled();
    });
});

describe("MultiLineComposer", () => {
    it("allows editing multi-line scripts and executing via button or Ctrl+Enter", () => {
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

        const textarea = screen.getByLabelText("Multi-line Eldritch script editor") as HTMLTextAreaElement;
        expect(textarea.value).toContain("process.list()");

        fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });
        expect(onRunCode).toHaveBeenCalledWith('for p in process.list():\n    print(p["name"])');
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

    it("renders live Knowledge Graph Intel drawer and stages credentials into SSH Pivot and Composer", () => {
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
                        hostFiles={[]}
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

        expect(screen.getByText("Knowledge Graph Intel")).toBeInTheDocument();
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
