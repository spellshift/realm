import React, { useEffect } from "react";
import { Minus, Pencil, RotateCcw } from "lucide-react";

export interface TabContextMenuProps {
    isOpen: boolean;
    x: number;
    y: number;
    tabLabel: string;
    hasCustomName?: boolean;
    onClose: () => void;
    onRename: () => void;
    onResetName?: () => void;
    onCloseTab?: () => void;
}

const TabContextMenu: React.FC<TabContextMenuProps> = ({
    isOpen,
    x,
    y,
    tabLabel,
    hasCustomName = false,
    onClose,
    onRename,
    onResetName,
    onCloseTab,
}) => {
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                onClose();
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    const menuWidth = 160;
    const menuHeight = hasCustomName ? 120 : 90;
    const posX = Math.max(
        8,
        Math.min(x, (typeof window !== "undefined" ? window.innerWidth : 1000) - menuWidth - 8)
    );
    const posY = Math.max(
        8,
        Math.min(y, (typeof window !== "undefined" ? window.innerHeight : 800) - menuHeight - 8)
    );

    return (
        <div
            className="fixed inset-0 z-50"
            onClick={onClose}
            onContextMenu={(e) => {
                e.preventDefault();
                onClose();
            }}
        >
            <div
                role="menu"
                aria-label={`Tab options for ${tabLabel}`}
                style={{ left: posX, top: posY }}
                className="fixed z-50 min-w-[150px] bg-[#181b24] border border-[#2d3345] rounded-md shadow-2xl p-1 text-xs select-none"
                onClick={(e) => e.stopPropagation()}
            >
                <button
                    type="button"
                    role="menuitem"
                    onClick={() => {
                        onClose();
                        onRename();
                    }}
                    className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-gray-200 hover:text-white hover:bg-purple-950/40 hover:text-purple-200 transition-colors text-left"
                >
                    <Pencil className="w-3.5 h-3.5 text-gray-400" />
                    <span>Rename</span>
                </button>

                {hasCustomName && onResetName && (
                    <button
                        type="button"
                        role="menuitem"
                        onClick={() => {
                            onClose();
                            onResetName();
                        }}
                        className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-gray-200 hover:text-white hover:bg-gray-800 transition-colors text-left"
                    >
                        <RotateCcw className="w-3.5 h-3.5 text-gray-400" />
                        <span>Reset Name</span>
                    </button>
                )}

                {onCloseTab && (
                    <>
                        <div className="border-t border-[#2d3345] my-1" />
                        <button
                            type="button"
                            role="menuitem"
                            onClick={() => {
                                onClose();
                                onCloseTab();
                            }}
                            className="w-full flex items-center gap-2 px-2.5 py-1.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors text-left"
                        >
                            <Minus className="w-3.5 h-3.5" />
                            <span>Close Tab</span>
                        </button>
                    </>
                )}
            </div>
        </div>
    );
};

export default TabContextMenu;
