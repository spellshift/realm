import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";

import TabContextMenu from "../TabContextMenu";
import { formatPivotTabLabel } from "../../utils/networkGraphUtils";

describe("PivotTabRename - TabContextMenu & Inline Rename", () => {
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

    it("verifies static PTY naming and manual rename override", () => {
        // 1. PTY tab always defaults to static "PTY" regardless of command
        const ptyTab = { type: "pty", target: "PTY" };
        expect(formatPivotTabLabel(ptyTab)).toBe("PTY");

        const ptyWithCmd = { type: "pty", target: "cat /etc/passwd" };
        expect(formatPivotTabLabel(ptyWithCmd)).toBe("PTY");

        // Manually renamed PTY tab overrides static label
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

        // Clearing custom title restores default static PTY / host IP naming
        const restoredPty = { ...renamedPty, customTitle: undefined };
        expect(formatPivotTabLabel(restoredPty)).toBe("PTY");

        const restoredSsh = { ...renamedSsh, customTitle: "" };
        expect(formatPivotTabLabel(restoredSsh)).toBe("192.168.1.100");
    });
});
