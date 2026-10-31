import React from "react";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";

import TabContextMenu from "../TabContextMenu";
import RenameTabModal from "../RenameTabModal";
import { formatPivotTabLabel } from "../../utils/networkGraphUtils";

describe("PivotTabRename - TabContextMenu & RenameTabModal", () => {
    it("renders TabContextMenu with Rename and Close Tab options", () => {
        const onRename = vi.fn();
        const onClose = vi.fn();
        const onCloseTab = vi.fn();

        render(
            <TabContextMenu
                isOpen={true}
                x={100}
                y={150}
                tabLabel="PTY"
                hasCustomName={false}
                onClose={onClose}
                onRename={onRename}
                onCloseTab={onCloseTab}
            />
        );

        expect(screen.getByRole("menu")).toBeInTheDocument();
        expect(screen.getByRole("menuitem", { name: /rename/i })).toBeInTheDocument();
        expect(screen.getByRole("menuitem", { name: /close tab/i })).toBeInTheDocument();
        expect(screen.queryByRole("menuitem", { name: /reset name/i })).not.toBeInTheDocument();

        fireEvent.click(screen.getByRole("menuitem", { name: /rename/i }));
        expect(onClose).toHaveBeenCalled();
        expect(onRename).toHaveBeenCalled();
    });

    it("renders Reset Name in TabContextMenu when hasCustomName is true", () => {
        const onResetName = vi.fn();
        const onClose = vi.fn();

        render(
            <TabContextMenu
                isOpen={true}
                x={100}
                y={150}
                tabLabel="My Custom Tab"
                hasCustomName={true}
                onClose={onClose}
                onRename={vi.fn()}
                onResetName={onResetName}
            />
        );

        expect(screen.getByRole("menuitem", { name: /reset name/i })).toBeInTheDocument();
        fireEvent.click(screen.getByRole("menuitem", { name: /reset name/i }));
        expect(onClose).toHaveBeenCalled();
        expect(onResetName).toHaveBeenCalled();
    });

    it("closes TabContextMenu when Escape key is pressed", () => {
        const onClose = vi.fn();

        render(
            <TabContextMenu
                isOpen={true}
                x={100}
                y={150}
                tabLabel="PTY"
                onClose={onClose}
                onRename={vi.fn()}
            />
        );

        fireEvent.keyDown(window, { key: "Escape" });
        expect(onClose).toHaveBeenCalled();
    });

    it("renders RenameTabModal and allows renaming PTY and SSH tabs", () => {
        const onRename = vi.fn();
        const onClose = vi.fn();

        const { rerender } = render(
            <RenameTabModal
                isOpen={true}
                onClose={onClose}
                currentName="PTY"
                tabType="pty"
                hasCustomName={false}
                onRename={onRename}
            />
        );

        expect(screen.getByText("Rename Tab")).toBeInTheDocument();
        expect(screen.getByText("PTY Pivot Tab")).toBeInTheDocument();

        const input = screen.getByTestId("rename-tab-input");
        expect(input).toHaveValue("PTY");

        act(() => {
            fireEvent.change(input, { target: { value: "Database Root Shell" } });
            fireEvent.click(screen.getByRole("button", { name: "Save" }));
        });

        expect(onRename).toHaveBeenCalledWith("Database Root Shell");
        expect(onClose).toHaveBeenCalled();

        // Rerender for SSH tab
        rerender(
            <RenameTabModal
                isOpen={true}
                onClose={onClose}
                currentName="10.0.0.1"
                tabType="ssh"
                hasCustomName={true}
                onRename={onRename}
                onResetName={vi.fn()}
            />
        );

        expect(screen.getByText("SSH Pivot Tab")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Reset to Auto" })).toBeInTheDocument();
    });

    it("resets custom name in RenameTabModal when clicking Reset to Auto", () => {
        const onResetName = vi.fn();
        const onClose = vi.fn();

        render(
            <RenameTabModal
                isOpen={true}
                onClose={onClose}
                currentName="Custom Name"
                tabType="pty"
                hasCustomName={true}
                onRename={vi.fn()}
                onResetName={onResetName}
            />
        );

        act(() => {
            fireEvent.click(screen.getByRole("button", { name: "Reset to Auto" }));
        });
        expect(onResetName).toHaveBeenCalled();
        expect(onClose).toHaveBeenCalled();
    });

    it("verifies manual rename strictly overrides PTY and SSH auto-naming", () => {
        // 1. PTY tab default auto-naming
        const ptyTab = { type: "pty", target: "PTY" };
        expect(formatPivotTabLabel(ptyTab)).toBe("PTY");

        // PTY tab auto-naming updates from executed command
        const ptyWithCmd = { type: "pty", target: "cat /etc/passwd" };
        expect(formatPivotTabLabel(ptyWithCmd)).toBe("cat /etc/passwd");

        // Manually renamed PTY tab overrides any auto-named command
        const renamedPty = {
            type: "pty",
            target: "cat /etc/passwd",
            customTitle: "Security Audit",
        };
        expect(formatPivotTabLabel(renamedPty)).toBe("Security Audit");

        // Subsequent command in PTY does not change the label when customTitle is set
        const renamedPtyNextCmd = {
            type: "pty",
            target: "find / -perm -4000",
            customTitle: "Security Audit",
        };
        expect(formatPivotTabLabel(renamedPtyNextCmd)).toBe("Security Audit");

        // 2. SSH tab default auto-naming extracts host/IP
        const sshTab = { type: "ssh", target: "admin:secret@192.168.1.100:22" };
        expect(formatPivotTabLabel(sshTab)).toBe("192.168.1.100");

        // Manually renamed SSH tab overrides host auto-naming
        const renamedSsh = {
            type: "ssh",
            target: "admin:secret@192.168.1.100:22",
            customTitle: "Jumpbox Alpha",
        };
        expect(formatPivotTabLabel(renamedSsh)).toBe("Jumpbox Alpha");

        // Clearing custom title restores auto-naming
        const restoredPty = { ...renamedPty, customTitle: undefined };
        expect(formatPivotTabLabel(restoredPty)).toBe("cat /etc/passwd");

        const restoredSsh = { ...renamedSsh, customTitle: "" };
        expect(formatPivotTabLabel(restoredSsh)).toBe("192.168.1.100");
    });
});
