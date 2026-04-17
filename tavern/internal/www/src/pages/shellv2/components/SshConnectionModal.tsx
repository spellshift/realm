import React, { Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import { XMarkIcon } from "@heroicons/react/24/outline";

interface SshConnectionModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConnect: (target: string) => void;
}

const SshConnectionModal: React.FC<SshConnectionModalProps> = ({ isOpen, onClose, onConnect }) => {
    const [target, setTarget] = useState("");

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (target.trim()) {
            onConnect(target.trim());
            setTarget("");
            onClose();
        }
    };

    const handleClose = () => {
        setTarget("");
        onClose();
    };

    return (
        <Transition show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={handleClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-200"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-150"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black/40" aria-hidden="true" />
                </Transition.Child>

                <div className="fixed inset-0 flex items-center justify-center p-4">
                    <Transition.Child
                        as={Fragment}
                        enter="ease-out duration-200"
                        enterFrom="opacity-0 scale-95"
                        enterTo="opacity-100 scale-100"
                        leave="ease-in duration-150"
                        leaveFrom="opacity-100 scale-100"
                        leaveTo="opacity-0 scale-95"
                    >
                        <Dialog.Panel className="w-full max-w-md rounded-lg bg-[#2d2d2d] border border-[#444] shadow-xl">
                            <div className="flex items-center justify-between px-5 py-4 border-b border-[#444]">
                                <Dialog.Title className="text-white font-semibold text-lg">
                                    SSH Connection
                                </Dialog.Title>
                                <button
                                    onClick={handleClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-5 w-5" />
                                </button>
                            </div>

                            <form onSubmit={handleSubmit} className="px-5 py-4">
                                <div className="mb-4">
                                    <label htmlFor="ssh-target" className="block text-sm font-medium text-gray-300 mb-2">
                                        Target
                                    </label>
                                    <input
                                        id="ssh-target"
                                        type="text"
                                        value={target}
                                        onChange={(e) => setTarget(e.target.value)}
                                        placeholder="user@hostname"
                                        className="w-full px-3 py-2 bg-[#1e1e1e] border border-[#555] rounded-md text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
                                        autoFocus
                                    />
                                    <p className="mt-1 text-xs text-gray-500">
                                        The SSH destination to connect to via the active portal.
                                    </p>
                                </div>

                                <div className="flex justify-end gap-2">
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "gray", size: "sm" }}
                                        onClick={handleClose}
                                    >
                                        Cancel
                                    </Button>
                                    <Button
                                        type="submit"
                                        buttonVariant="solid"
                                        buttonStyle={{ color: "purple", size: "sm" }}
                                        disabled={!target.trim()}
                                    >
                                        Connect
                                    </Button>
                                </div>
                            </form>
                        </Dialog.Panel>
                    </Transition.Child>
                </div>
            </Dialog>
        </Transition>
    );
};

export default SshConnectionModal;
