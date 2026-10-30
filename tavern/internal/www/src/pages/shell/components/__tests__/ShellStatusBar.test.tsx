import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import ShellStatusBar from "../ShellStatusBar";
import { ConnectionStatus } from "../../../../lib/browser-adapter";
import React from "react";
import { expect, describe, it } from "vitest";
import "@testing-library/jest-dom";

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

    it("displays portal active message when portalId is present without showing portal ID, and renders cogwheel toggle", () => {
        render(<ShellStatusBar {...defaultProps} portalId={123} />);
        expect(screen.getByText("Portal Active")).toBeInTheDocument();
        expect(screen.queryByText("(ID: 123)")).not.toBeInTheDocument();
        expect(screen.getByTestId("session-settings-toggle")).toBeInTheDocument();
    });

    it("displays async message when portalId is null", () => {
        render(<ShellStatusBar {...defaultProps} portalId={null} />);
        expect(screen.getByText("async")).toBeInTheDocument();
        expect(screen.getByTestId("session-settings-toggle")).toBeInTheDocument();
    });

    it("displays callback timer with 'Next' label", () => {
        render(<ShellStatusBar {...defaultProps} timeUntilCallback="in 5 minutes" />);
        expect(screen.getByText("Next")).toBeInTheDocument();
        expect(screen.queryByText("Next callback:")).not.toBeInTheDocument();
        expect(screen.getByText("in 5 minutes")).toBeInTheDocument();
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
