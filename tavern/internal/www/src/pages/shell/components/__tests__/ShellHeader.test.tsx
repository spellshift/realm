import { render, screen, fireEvent } from "@testing-library/react";
import ShellHeader from "../ShellHeader";
import React from "react";
import { expect, describe, it, vi, beforeAll } from "vitest";
import { BrowserRouter } from "react-router-dom";
import "@testing-library/jest-dom";

// Polyfill ResizeObserver for Headless UI Menu in jsdom
beforeAll(() => {
    if (typeof globalThis.ResizeObserver === "undefined") {
        globalThis.ResizeObserver = class {
            observe() {}
            unobserve() {}
            disconnect() {}
        } as unknown as typeof ResizeObserver;
    }
});

// Mock Tooltip because it often causes issues in tests
vi.mock("@chakra-ui/react", async () => {
    const actual = await vi.importActual("@chakra-ui/react");
    return {
        ...actual,
        Tooltip: ({ children, label }: any) => <div title={typeof label === 'string' ? label : "tooltip"}>{children}</div>,
    };
});

// Mock NotificationBell to avoid requiring ApolloProvider in ShellHeader tests
vi.mock("../../../../components/notifications/NotificationBell", () => ({
    default: () => <div data-testid="notification-bell" />,
}));

describe("ShellHeader", () => {
    const mockShellData = {
        node: {
            beacon: {
                name: "test-beacon",
                principal: "root",
                agentIdentifier: "agent-1",
                interval: 60,
                transport: "HTTP",
                host: {
                    id: "host-1",
                    name: "test-host",
                    primaryIP: "192.168.1.1",
                    externalIP: "8.8.8.8",
                    platform: "linux",
                    tags: { edges: [] }
                }
            }
        }
    };

    const defaultProps = {
        shellData: mockShellData,
        portalId: null as number | null,
        onExport: vi.fn(),
        onNewPortal: vi.fn(),
        onSshConnect: vi.fn(),
        onPtyOpen: vi.fn(),
        onSendCtrlC: vi.fn(),
        onSendCtrlR: vi.fn(),
        onClosePortal: vi.fn(),
    };

    it("renders beacon name and host name", () => {
        render(
            <BrowserRouter>
                <ShellHeader {...defaultProps} />
            </BrowserRouter>
        );
        expect(screen.getByText("test-beacon")).toBeInTheDocument();
        expect(screen.getByText("test-host")).toBeInTheDocument();
    });

    it("renders principal badge when present and hides user icon on mobile to prevent wrapping", () => {
        render(
            <BrowserRouter>
                <ShellHeader {...defaultProps} />
            </BrowserRouter>
        );
        expect(screen.getByText("root")).toBeInTheDocument();
        const userIcon = screen.getByTestId("shell-breadcrumb-user-icon");
        expect(userIcon).toBeInTheDocument();
        expect(userIcon).toHaveClass("hidden");
        expect(userIcon).toHaveClass("sm:block");
    });

    it("does not render principal badge when absent", () => {
        const dataWithoutPrincipal = {
            node: {
                ...mockShellData.node,
                beacon: {
                    ...mockShellData.node.beacon,
                    principal: null
                }
            }
        };
        render(
            <BrowserRouter>
                <ShellHeader {...defaultProps} shellData={dataWithoutPrincipal} />
            </BrowserRouter>
        );
        expect(screen.queryByText("root")).not.toBeInTheDocument();
    });

    it("calls onExport when export menu item is clicked and omits redundant toolbar entries from actions menu", () => {
        const onExport = vi.fn();
        render(
            <BrowserRouter>
                <ShellHeader
                    {...defaultProps}
                    onExport={onExport}
                    onToggleComposer={vi.fn()}
                    onToggleKnowledge={vi.fn()}
                    onTogglePivots={vi.fn()}
                    onToggleNux={vi.fn()}
                />
            </BrowserRouter>
        );
        // Open the actions menu first
        const actionsButton = screen.getByLabelText("Shell actions");
        fireEvent.click(actionsButton);

        // Redundant toolbar items should not appear in the ... menu
        expect(screen.queryByText("Knowledge Graph")).not.toBeInTheDocument();
        expect(screen.queryByText("History")).not.toBeInTheDocument();
        expect(screen.queryByText("Composer")).not.toBeInTheDocument();
        expect(screen.queryByText("Operator Guide & Docs")).not.toBeInTheDocument();

        const exportButton = screen.getByText("Export");
        fireEvent.click(exportButton);
        expect(onExport).toHaveBeenCalled();
    });

    it("renders New Interpreter, Replay Tutorial, Documentation link, and Report a Bug in the actions menu while omitting portal/pty/ssh options", () => {
        const onSpawnFreshShell = vi.fn();
        const onReplayTutorial = vi.fn();
        const onNewPortal = vi.fn();
        render(
            <BrowserRouter>
                <ShellHeader
                    {...defaultProps}
                    onSpawnFreshShell={onSpawnFreshShell}
                    onReplayTutorial={onReplayTutorial}
                    onNewPortal={onNewPortal}
                    actionsMenuTourOpen={true}
                />
            </BrowserRouter>
        );

        const newInterpreterBtn = screen.getByText("New Interpreter").closest("button")!;
        const replayTutorialBtn = screen.getByText("Replay Tutorial").closest("button")!;
        const docsLink = screen.getByLabelText("Documentation");
        const reportBugLink = screen.getByLabelText("Report a bug");
        expect(newInterpreterBtn).toHaveClass("text-white");
        expect(replayTutorialBtn).toHaveClass("text-white");
        fireEvent.click(replayTutorialBtn);
        expect(onReplayTutorial).toHaveBeenCalledTimes(1);
        expect(docsLink).toHaveAttribute(
            "href",
            "https://docs.realm.pub/user-guide/eldritch"
        );
        expect(screen.getByText("Documentation")).toBeInTheDocument();
        expect(reportBugLink).toHaveAttribute(
            "href",
            "https://github.com/spellshift/realm/issues/new?template=bug_report.md"
        );
        expect(screen.getByText("Report a Bug")).toBeInTheDocument();

        // Redundant portal, pty, and ssh options should not appear in the ... menu
        expect(screen.queryByText("New Portal")).not.toBeInTheDocument();
        expect(screen.queryByText("Close Portal")).not.toBeInTheDocument();
        expect(screen.queryByText("SSH")).not.toBeInTheDocument();
        expect(screen.queryByText("PTY")).not.toBeInTheDocument();
    });

    it("renders unified toolbar background, active button highlight, and badges only for active pivots and active queue", () => {
        const onToggleComposer = vi.fn();
        const onToggleKnowledge = vi.fn();
        const onTogglePivots = vi.fn();
        const onToggleHistory = vi.fn();

        const { rerender } = render(
            <BrowserRouter>
                <ShellHeader
                    {...defaultProps}
                    activeUsers={[{ id: "u1", name: "operator1" }]}
                    composerOpen={true}
                    onToggleComposer={onToggleComposer}
                    onToggleKnowledge={onToggleKnowledge}
                    credentialCount={3}
                    onTogglePivots={onTogglePivots}
                    pivotCount={2}
                    onToggleHistory={onToggleHistory}
                    historyCount={12}
                    activeQueueCount={0}
                />
            </BrowserRouter>
        );

        const toolbar = screen.getByTestId("shell-header-toolbar");
        expect(toolbar).toBeInTheDocument();
        expect(screen.getByTestId("shell-header-users-divider")).toBeInTheDocument();
        const sidebarToggleBtn = screen.getByTestId("status-bar-sidebar-toggle");
        const homeLink = screen.getByTitle("Return Home");
        expect(sidebarToggleBtn).toBeInTheDocument();
        expect(sidebarToggleBtn.compareDocumentPosition(homeLink) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
        expect(homeLink).not.toHaveClass("border");

        const knowledgeBtn = screen.getByLabelText("Toggle knowledge graph");
        expect(knowledgeBtn).toHaveClass("bg-transparent");
        fireEvent.click(knowledgeBtn);
        expect(onToggleKnowledge).toHaveBeenCalled();
        expect(screen.getByText("3")).toBeInTheDocument();

        const pivotsBtn = screen.getByLabelText("Toggle pivots");
        fireEvent.click(pivotsBtn);
        expect(onTogglePivots).toHaveBeenCalled();
        expect(screen.getByText("2")).toBeInTheDocument();

        const historyBtn = screen.getByLabelText("Toggle command history");
        fireEvent.click(historyBtn);
        expect(onToggleHistory).toHaveBeenCalled();
        // Completed history count (12) should NOT display when activeQueueCount is 0
        expect(screen.queryByText("12")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Toggle operator guide")).not.toBeInTheDocument();
        expect(screen.queryByLabelText("Toggle multi-line composer")).not.toBeInTheDocument();

        // When activeQueueCount > 0, History button displays the active queue badge
        rerender(
            <BrowserRouter>
                <ShellHeader
                    {...defaultProps}
                    onToggleHistory={onToggleHistory}
                    historyCount={12}
                    activeQueueCount={4}
                />
            </BrowserRouter>
        );
        expect(screen.getByText("4")).toBeInTheDocument();
    });
});
