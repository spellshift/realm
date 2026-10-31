import React from "react";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";

import TabContextMenu from "../TabContextMenu";
import GraphPivotsSection from "../graph/GraphPivotsSection";
import { formatPivotTabLabel } from "../../utils/networkGraphUtils";
import { ShellPivotItem } from "../../hooks/useShellData";

describe("PivotTabRename - TabContextMenu, Context Drawer & Inline Rename", () => {
    it("renders TabContextMenu with Rename and Minimize options", () => {
        const onRename = vi.fn();
        const onClose = vi.fn();
        const onMinimize = vi.fn();

        render(
            <TabContextMenu
                isOpen={true}
                x={100}
                y={150}
                tabLabel="PTY"
                hasCustomName={false}
                onClose={onClose}
                onRename={onRename}
                onMinimize={onMinimize}
            />
        );

        expect(screen.getByRole("menu")).toBeInTheDocument();
        expect(screen.getByRole("menuitem", { name: /rename/i })).toBeInTheDocument();
        expect(screen.getByRole("menuitem", { name: /minimize/i })).toBeInTheDocument();
        expect(screen.queryByRole("menuitem", { name: /close tab/i })).not.toBeInTheDocument();
        expect(screen.queryByRole("menuitem", { name: /reset name/i })).not.toBeInTheDocument();

        fireEvent.click(screen.getByRole("menuitem", { name: /rename/i }));
        expect(onClose).toHaveBeenCalled();
        expect(onRename).toHaveBeenCalled();

        fireEvent.click(screen.getByRole("menuitem", { name: /minimize/i }));
        expect(onMinimize).toHaveBeenCalled();
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

    it("allows renaming pivot tabs directly from the context drawer (GraphPivotsSection)", () => {
        const onRenamePivot = vi.fn();
        const onResetPivotName = vi.fn();
        const pivots: ShellPivotItem[] = [
            {
                id: "p1",
                kind: "ssh",
                destination: "admin@10.0.0.1:22",
                activeTarget: "admin@10.0.0.1:22",
            },
        ];

        const { rerender } = render(
            <GraphPivotsSection
                portalId={1}
                pivots={pivots}
                onRenamePivot={onRenamePivot}
                onResetPivotName={onResetPivotName}
            />
        );

        // Initial view shows auto-named label
        expect(screen.getByText("10.0.0.1")).toBeInTheDocument();

        // Clicking the rename pencil icon switches to inline input
        const renameBtn = screen.getByRole("button", { name: /rename pivot 10\.0\.0\.1/i });
        fireEvent.click(renameBtn);

        const input = screen.getByTestId("rename-pivot-input-p1");
        expect(input).toBeInTheDocument();
        expect(input).toHaveValue("10.0.0.1");

        // Type a new title and submit with Enter
        fireEvent.change(input, { target: { value: "Prod Database Bastion" } });
        fireEvent.keyDown(input, { key: "Enter" });

        expect(onRenamePivot).toHaveBeenCalledWith("p1", "Prod Database Bastion");

        // Rerender with customTitle set
        const updatedPivots: ShellPivotItem[] = [
            {
                ...pivots[0],
                customTitle: "Prod Database Bastion",
            },
        ];

        rerender(
            <GraphPivotsSection
                portalId={1}
                pivots={updatedPivots}
                onRenamePivot={onRenamePivot}
                onResetPivotName={onResetPivotName}
            />
        );

        expect(screen.getByText("Prod Database Bastion")).toBeInTheDocument();

        // Reset button should now be visible
        const resetBtn = screen.getByRole("button", {
            name: /reset name for prod database bastion/i,
        });
        expect(resetBtn).toBeInTheDocument();
        fireEvent.click(resetBtn);

        expect(onResetPivotName).toHaveBeenCalledWith("p1");
    });

    it("verifies double-click and escape cancellation in context drawer inline rename", () => {
        const onRenamePivot = vi.fn();
        const pivots: ShellPivotItem[] = [
            {
                id: "pty-1",
                kind: "pty",
                destination: "PTY",
                activeTarget: "PTY",
            },
        ];

        render(
            <GraphPivotsSection
                portalId={1}
                pivots={pivots}
                onRenamePivot={onRenamePivot}
            />
        );

        const label = screen.getByTitle("Double-click to rename");
        fireEvent.doubleClick(label);

        const input = screen.getByTestId("rename-pivot-input-pty-1");
        expect(input).toBeInTheDocument();

        // Change text but press Escape
        fireEvent.change(input, { target: { value: "Unsaved Name" } });
        fireEvent.keyDown(input, { key: "Escape" });

        expect(onRenamePivot).not.toHaveBeenCalled();
        expect(screen.queryByTestId("rename-pivot-input-pty-1")).not.toBeInTheDocument();
        expect(screen.getByTitle("Double-click to rename")).toHaveTextContent("PTY");
    });

    it("supports inline editing directly on the tab bar via double-click and blur/enter", () => {
        const TabBarTest = () => {
            const [tab, setTab] = React.useState({
                id: "tab-1",
                type: "ssh",
                target: "root@192.168.1.50:22",
                customTitle: undefined as string | undefined,
            });
            const [isEditing, setIsEditing] = React.useState(false);
            const [editingTitle, setEditingTitle] = React.useState("");

            const tabLabel = formatPivotTabLabel(tab);

            return (
                <div>
                    <div
                        data-testid="tab-element"
                        onDoubleClick={() => {
                            setEditingTitle(tab.customTitle || tabLabel);
                            setIsEditing(true);
                        }}
                    >
                        {isEditing ? (
                            <input
                                data-testid="inline-rename-tab-input"
                                value={editingTitle}
                                onChange={(e) => setEditingTitle(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === "Enter") {
                                        setTab((prev) => ({
                                            ...prev,
                                            customTitle: editingTitle.trim() || undefined,
                                        }));
                                        setIsEditing(false);
                                    } else if (e.key === "Escape") {
                                        setIsEditing(false);
                                    }
                                }}
                                onBlur={() => {
                                    setTab((prev) => ({
                                        ...prev,
                                        customTitle: editingTitle.trim() || undefined,
                                    }));
                                    setIsEditing(false);
                                }}
                            />
                        ) : (
                            <span data-testid="tab-label">{tabLabel}</span>
                        )}
                        <button aria-label={`Minimize ${tabLabel} tab`}>-</button>
                    </div>
                </div>
            );
        };

        render(<TabBarTest />);

        expect(screen.getByTestId("tab-label")).toHaveTextContent("192.168.1.50");
        expect(screen.getByRole("button", { name: "Minimize 192.168.1.50 tab" })).toBeInTheDocument();

        // Double click tab to edit inline
        fireEvent.doubleClick(screen.getByTestId("tab-element"));

        const input = screen.getByTestId("inline-rename-tab-input");
        expect(input).toBeInTheDocument();
        expect(input).toHaveValue("192.168.1.50");

        // Type new name and Enter
        fireEvent.change(input, { target: { value: "Gateway" } });
        fireEvent.keyDown(input, { key: "Enter" });

        expect(screen.queryByTestId("inline-rename-tab-input")).not.toBeInTheDocument();
        expect(screen.getByTestId("tab-label")).toHaveTextContent("Gateway");
        expect(screen.getByRole("button", { name: "Minimize Gateway tab" })).toBeInTheDocument();
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
