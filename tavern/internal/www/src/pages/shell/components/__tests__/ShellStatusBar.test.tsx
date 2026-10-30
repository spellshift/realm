import { render, screen, fireEvent } from "@testing-library/react";
import ShellStatusBar from "../ShellStatusBar";
import { ConnectionStatus } from "../../../../lib/browser-adapter";
import React from "react";
import { expect, describe, it, vi } from "vitest";
import "@testing-library/jest-dom";

// Mock NotificationBell to avoid requiring ApolloProvider in ShellStatusBar tests
vi.mock("../../../../components/notifications/NotificationBell", () => ({
    default: () => <div data-testid="notification-bell" />,
}));

describe("ShellStatusBar", () => {
    const defaultProps = {
        portalId: null,
        timeUntilCallback: "",
        isMissedCallback: false,
        connectionStatus: "connected" as ConnectionStatus,
        connectionMessage: "",
    };

    it("renders connected status correctly", () => {
        const { container } = render(<ShellStatusBar {...defaultProps} connectionStatus="connected" />);
        // Wifi icon should be present (green)
        const connectedIcon = container.querySelector(".text-green-500");
        expect(connectedIcon).toBeInTheDocument();
    });

    it("renders disconnected status correctly", () => {
        const { container } = render(<ShellStatusBar {...defaultProps} connectionStatus="disconnected" />);
        const disconnectedIcon = container.querySelector(".text-red-500");
        expect(disconnectedIcon).toBeInTheDocument();
    });

    it("renders reconnecting status correctly", () => {
        const { container } = render(<ShellStatusBar {...defaultProps} connectionStatus="reconnecting" />);
        const reconnectingIcon = container.querySelector(".text-yellow-500.animate-spin");
        expect(reconnectingIcon).toBeInTheDocument();
    });

    it("displays portal active message when portalId is present without showing portal ID, and opens Close Portal warning dialog when clicked", () => {
        const onClosePortal = vi.fn();
        render(
            <ShellStatusBar
                {...defaultProps}
                portalId={123}
                onClosePortal={onClosePortal}
                onToggleComposer={vi.fn()}
                composerOpen={true}
            />
        );
        expect(screen.getByText("Portal Active")).toBeInTheDocument();
        expect(screen.queryByText("(ID: 123)")).not.toBeInTheDocument();
        const bellEl = screen.getByTestId("notification-bell");
        const settingsEl = screen.getByTestId("session-settings-toggle");
        const composerEl = screen.getByTestId("status-bar-composer-toggle");
        expect(bellEl).toBeInTheDocument();
        expect(settingsEl).toBeInTheDocument();
        expect(composerEl).toBeInTheDocument();

        // Verify left-toolbar DOM order: Notification Bell -> Settings -> Composer
        expect(bellEl.compareDocumentPosition(settingsEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
        expect(settingsEl.compareDocumentPosition(composerEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

        fireEvent.click(screen.getByTestId("status-bar-portal-active-button"));
        const closeDialog = screen.getByTestId("close-portal-dialog");
        expect(closeDialog).toBeInTheDocument();
        expect(closeDialog).toHaveTextContent(/close all associated pivots/i);

        fireEvent.click(screen.getByRole("button", { name: "Close Portal" }));
        expect(onClosePortal).toHaveBeenCalledTimes(1);
    });

    it("displays async message when portalId is null and opens Open Portal dialog when clicked", () => {
        const onNewPortal = vi.fn();
        render(<ShellStatusBar {...defaultProps} portalId={null} onNewPortal={onNewPortal} />);
        expect(screen.getByText("async")).toBeInTheDocument();
        expect(screen.getByTestId("session-settings-toggle")).toBeInTheDocument();

        fireEvent.click(screen.getByTestId("status-bar-async-button"));
        const openDialog = screen.getByTestId("open-portal-dialog");
        expect(openDialog).toBeInTheDocument();
        expect(openDialog).toHaveTextContent(
            "Opens an interactive portal on this beacon, which works similar to a SOCKS5 proxy. This establishes a real-time stream between the agent and server for low-latency execution, interactive PTY, or pivoting operations."
        );

        fireEvent.click(screen.getByRole("button", { name: "Open Portal" }));
        expect(onNewPortal).toHaveBeenCalledTimes(1);
    });

    it("displays callback timer with 'Next' label and shows callback stats popover on hover", () => {
        render(
            <ShellStatusBar
                {...defaultProps}
                timeUntilCallback="in 5 minutes"
                callbackStats={{
                    intervalSeconds: 5,
                    formattedInterval: "5s",
                    sampleCount: 100,
                    onTimeCount: 98,
                    lateCount: 2,
                    onTimePercentage: 98,
                    avgLatencyMs: 85,
                    lastSeenAt: "2025-01-01T12:00:00Z",
                    nextSeenAt: "2025-01-01T12:00:05Z",
                }}
            />
        );
        expect(screen.getByText("Next")).toBeInTheDocument();
        expect(screen.queryByText("Next callback:")).not.toBeInTheDocument();
        expect(screen.getByText("in 5 minutes")).toBeInTheDocument();
        expect(screen.queryByTestId("status-bar-callback-popover")).not.toBeInTheDocument();

        const nextCallbackEl = screen.getByTestId("status-bar-next-callback");
        fireEvent.mouseEnter(nextCallbackEl);

        const popover = screen.getByTestId("status-bar-callback-popover");
        expect(popover).toBeInTheDocument();
        expect(screen.getByTestId("callback-stats-interval")).toHaveTextContent("5s");
        expect(screen.getByTestId("callback-stats-ontime")).toHaveTextContent("98 / 100 (98%)");

        fireEvent.mouseLeave(nextCallbackEl);
        expect(screen.queryByTestId("status-bar-callback-popover")).not.toBeInTheDocument();
    });

    it("highlights missed callback", () => {
        render(<ShellStatusBar {...defaultProps} timeUntilCallback="expected 5 minutes ago" isMissedCallback={true} />);
        const timer = screen.getByText("expected 5 minutes ago");
        expect(timer).toHaveClass("text-red-500 font-bold");
    });

    it("displays terminated status when closedAt is present", () => {
        render(<ShellStatusBar {...defaultProps} closedAt="2024-01-01T00:00:00Z" />);
        expect(screen.getByText(/Terminated/i)).toBeInTheDocument();
    });

    it("renders executing/queued spinner at bottom of screen with popover displaying currently running and queued commands", () => {
        render(
            <ShellStatusBar
                {...defaultProps}
                runningCount={1}
                queuedCount={2}
                runningTasks={[
                    {
                        id: "t-run",
                        createdAt: "2025-01-01T12:00:00Z",
                        claimedAt: "2025-01-01T12:00:02Z",
                        input: 'sys.shell("uname -a")',
                    },
                ]}
                queuedTasks={[
                    {
                        id: "t-q1",
                        createdAt: "2025-01-01T12:01:00Z",
                        input: 'file.list("/etc")',
                    },
                    {
                        id: "t-q2",
                        createdAt: "2025-01-01T12:02:00Z",
                        input: "process.list()",
                    },
                ]}
            />
        );
        const queueBtn = screen.getByTestId("status-bar-queue-button");
        expect(queueBtn).toHaveTextContent(/1 executing/i);
        expect(queueBtn).toHaveTextContent(/2 queued/i);
        expect(screen.queryByTestId("status-bar-queue-popover")).not.toBeInTheDocument();

        fireEvent.click(queueBtn);
        const popover = screen.getByTestId("status-bar-queue-popover");
        expect(popover).toBeInTheDocument();
        expect(popover).toHaveTextContent('sys.shell("uname -a")');
        expect(popover).toHaveTextContent('file.list("/etc")');
        expect(popover).toHaveTextContent("process.list()");
    });
});
