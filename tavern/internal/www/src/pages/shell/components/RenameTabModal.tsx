import React, { Fragment, useEffect, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import { Pencil } from "lucide-react";

export interface RenameTabModalProps {
    isOpen: boolean;
    onClose: () => void;
    currentName: string;
    tabType?: string;
    hasCustomName?: boolean;
    onRename: (newName: string) => void;
    onResetName?: () => void;
}

const RenameTabModal: React.FC<RenameTabModalProps> = ({
    isOpen,
    onClose,
    currentName,
    tabType = "PTY",
    hasCustomName = false,
    onRename,
    onResetName,
}) => {
    const [name, setName] = useState(currentName);

    useEffect(() => {
        if (isOpen) {
            setName(currentName);
        }
    }, [isOpen, currentName]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        const trimmed = name.trim();
        if (trimmed) {
            onRename(trimmed);
            onClose();
        } else if (onResetName) {
            onResetName();
            onClose();
        }
    };

    const handleReset = () => {
        onResetName?.();
        onClose();
    };

    return (
        <Transition appear show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={onClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-200"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-150"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm" />
                </Transition.Child>

                <div className="fixed inset-0 overflow-y-auto">
                    <div className="flex min-h-full items-center justify-center p-4 text-center">
                        <Transition.Child
                            as={Fragment}
                            enter="ease-out duration-200"
                            enterFrom="opacity-0 scale-95"
                            enterTo="opacity-100 scale-100"
                            leave="ease-in duration-150"
                            leaveFrom="opacity-100 scale-100"
                            leaveTo="opacity-0 scale-95"
                        >
                            <Dialog.Panel className="w-full max-w-sm transform overflow-hidden rounded-xl bg-[#181b24] border border-[#2d3345] p-5 text-left align-middle shadow-2xl transition-all">
                                <div className="flex items-center justify-between pb-3 border-b border-[#2d3345]">
                                    <div className="flex items-center gap-2">
                                        <div className="p-1.5 rounded-md bg-purple-950/50 border border-purple-800/40 text-purple-400">
                                            <Pencil size={15} />
                                        </div>
                                        <div>
                                            <Dialog.Title className="text-sm font-semibold text-white">
                                                Rename Tab
                                            </Dialog.Title>
                                            <p className="text-[11px] text-gray-400">
                                                {tabType.toUpperCase()} Pivot Tab
                                            </p>
                                        </div>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={onClose}
                                        aria-label="Close dialog"
                                        className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-[#252b3b] transition-colors"
                                    >
                                        <XMarkIcon className="w-5 h-5" />
                                    </button>
                                </div>

                                <form onSubmit={handleSubmit} className="mt-4 space-y-4">
                                    <div>
                                        <label
                                            htmlFor="tab-name-input"
                                            className="block text-xs font-medium text-gray-300 mb-1"
                                        >
                                            Tab Name
                                        </label>
                                        <input
                                            id="tab-name-input"
                                            data-testid="rename-tab-input"
                                            aria-label="Tab name"
                                            type="text"
                                            value={name}
                                            onChange={(e) => setName(e.target.value)}
                                            placeholder="Enter tab name..."
                                            autoFocus
                                            className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-xs text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        />
                                        <p className="mt-1.5 text-[11px] text-gray-400">
                                            Manually renaming overrides automatic naming from commands or destinations.
                                        </p>
                                    </div>

                                    <div className="flex items-center justify-between pt-1">
                                        <div>
                                            {hasCustomName && onResetName && (
                                                <button
                                                    type="button"
                                                    onClick={handleReset}
                                                    className="text-xs font-medium text-gray-400 hover:text-amber-400 py-1.5 px-2 rounded hover:bg-[#252b3b] transition-colors"
                                                >
                                                    Reset to Auto
                                                </button>
                                            )}
                                        </div>
                                        <div className="flex items-center gap-2">
                                            <button
                                                type="button"
                                                onClick={onClose}
                                                className="text-xs font-semibold py-1.5 px-3 rounded-md text-gray-300 hover:bg-[#2c3242] transition-colors"
                                            >
                                                Cancel
                                            </button>
                                            <button
                                                type="submit"
                                                disabled={!name.trim()}
                                                className="text-xs font-semibold py-1.5 px-3.5 rounded-md bg-purple-700 text-white hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                            >
                                                Save
                                            </button>
                                        </div>
                                    </div>
                                </form>
                            </Dialog.Panel>
                        </Transition.Child>
                    </div>
                </div>
            </Dialog>
        </Transition>
    );
};

export default RenameTabModal;
