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

    it("renders principal badge when present", () => {
        render(
            <BrowserRouter>
                <ShellHeader {...defaultProps} />
            </BrowserRouter>
        );
        expect(screen.getByText("root")).toBeInTheDocument();
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
                    onToggleHistory={vi.fn()}
                    onToggleNux={vi.fn()}
                />
            </BrowserRouter>
        );
        // Open the actions menu first
        const actionsButton = screen.getByLabelText("Shell actions");
        fireEvent.click(actionsButton);

        // Redundant toolbar items should not appear in the ... menu
        expect(screen.queryByText("Knowledge Graph")).not.toBeInTheDocument();
        expect(screen.queryByText("Pivots")).not.toBeInTheDocument();
        expect(screen.queryByText("History")).not.toBeInTheDocument();
        expect(screen.queryByText("Composer")).not.toBeInTheDocument();
        expect(screen.queryByText("Operator Guide & Docs")).not.toBeInTheDocument();

        const exportButton = screen.getByText("Export");
        fireEvent.click(exportButton);
        expect(onExport).toHaveBeenCalled();
    });

    it("renders unified toolbar background, active button highlight, and badges only for active pivots and active queue", () => {
        const onToggleComposer = vi.fn();
        const onToggleKnowledge = vi.fn();
        const onTogglePivots = vi.fn();
        const onToggleHistory = vi.fn();
        const onToggleNux = vi.fn();

        const { rerender } = render(
            <BrowserRouter>
                <ShellHeader
                    {...defaultProps}
                    composerOpen={true}
                    onToggleComposer={onToggleComposer}
                    onToggleKnowledge={onToggleKnowledge}
                    credentialCount={3}
                    onTogglePivots={onTogglePivots}
                    pivotCount={2}
                    onToggleHistory={onToggleHistory}
                    historyCount={12}
                    activeQueueCount={0}
                    onToggleNux={onToggleNux}
                />
            </BrowserRouter>
        );

        const toolbar = screen.getByTestId("shell-header-toolbar");
        expect(toolbar).toHaveClass("bg-[#1d222f]", "border", "border-[#2f364a]");

        const composerBtn = screen.getByLabelText("Toggle multi-line composer");
        expect(composerBtn).toHaveClass("bg-purple-600", "text-white");
        fireEvent.click(composerBtn);
        expect(onToggleComposer).toHaveBeenCalled();

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

        const guideBtn = screen.getByLabelText("Toggle operator guide");
        fireEvent.click(guideBtn);
        expect(onToggleNux).toHaveBeenCalled();

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
