import { render, screen } from "@testing-library/react";
import ShellStatusBar from "../ShellStatusBar";
import { ConnectionStatus } from "../../../../lib/headless-adapter";
import React from "react";
import { expect, describe, it } from "vitest";
import "@testing-library/jest-dom";

describe("ShellStatusBar", () => {
    const defaultProps = {
        portalId: null,
        timeUntilCallback: "",
        isMissedCallback: false,
        connectionStatus: "connected" as ConnectionStatus,
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

    it("displays portal active message when portalId is present", () => {
        render(<ShellStatusBar {...defaultProps} portalId={123} />);
        expect(screen.getByText("Portal Active (ID: 123)")).toBeInTheDocument();
    });

    it("displays non-interactive message when portalId is null", () => {
        render(<ShellStatusBar {...defaultProps} portalId={null} />);
        expect(screen.getByText("non-interactive")).toBeInTheDocument();
    });

    it("displays callback timer", () => {
        render(<ShellStatusBar {...defaultProps} timeUntilCallback="in 5 minutes" />);
        expect(screen.getByText("in 5 minutes")).toBeInTheDocument();
    });

    it("highlights missed callback", () => {
        render(<ShellStatusBar {...defaultProps} timeUntilCallback="expected 5 minutes ago" isMissedCallback={true} />);
        const timer = screen.getByText("expected 5 minutes ago");
        expect(timer).toHaveClass("text-red-500 font-bold");
    });
});
