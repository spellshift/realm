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
        onWinrmConnect: vi.fn(),
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

    it("calls onExport when export menu item is clicked", () => {
        const onExport = vi.fn();
        render(
            <BrowserRouter>
                <ShellHeader {...defaultProps} onExport={onExport} />
            </BrowserRouter>
        );
        // Open the actions menu first
        const actionsButton = screen.getByLabelText("Shell actions");
        fireEvent.click(actionsButton);
        const exportButton = screen.getByText("Export");
        fireEvent.click(exportButton);
        expect(onExport).toHaveBeenCalled();
    });
});
