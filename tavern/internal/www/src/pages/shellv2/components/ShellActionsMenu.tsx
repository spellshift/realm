import React, { Fragment, useState } from "react";
import { Menu, Transition } from "@headlessui/react";
import { ChevronDownIcon } from "@heroicons/react/20/solid";
import { Share, Terminal, Globe, SquareTerminal } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import SshConnectionModal from "./SshConnectionModal";

interface ShellActionsMenuProps {
    portalId: number | null;
    onExport: () => void;
    onNewPortal: () => void;
    onSshConnect: (target: string) => void;
    onPtyOpen: () => void;
}

const PORTAL_REQUIRED_TOOLTIP = "Requires an active portal connection. Create a portal first using pivot.create_portal().";

const ShellActionsMenu: React.FC<ShellActionsMenuProps> = ({
    portalId,
    onExport,
    onNewPortal,
    onSshConnect,
    onPtyOpen,
}) => {
    const [sshModalOpen, setSshModalOpen] = useState(false);
    const hasPortal = portalId !== null;

    return (
        <>
            <Menu as="div" className="relative">
                <Menu.Button
                    as={Button}
                    buttonVariant="solid"
                    buttonStyle={{ color: "gray", size: "sm" }}
                    rightIcon={<ChevronDownIcon className="h-4 w-4" aria-hidden="true" />}
                >
                    Actions
                </Menu.Button>
                <Transition
                    as={Fragment}
                    enter="transition ease-out duration-100"
                    enterFrom="transform opacity-0 scale-95"
                    enterTo="transform opacity-100 scale-100"
                    leave="transition ease-in duration-75"
                    leaveFrom="transform opacity-100 scale-100"
                    leaveTo="transform opacity-0 scale-95"
                >
                    <Menu.Items className="absolute left-0 mt-2 w-56 origin-top-left rounded-md bg-[#2d2d2d] border border-[#444] shadow-lg focus:outline-none z-50">
                        <div className="px-1 py-1">
                            <Menu.Item>
                                {() => (
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "gray", size: "sm" }}
                                        className="w-full justify-start text-gray-200 hover:bg-[#3d3d3d]"
                                        leftIcon={<Share className="w-4 h-4" />}
                                        onClick={onExport}
                                    >
                                        Export
                                    </Button>
                                )}
                            </Menu.Item>

                            <Menu.Item>
                                {() => (
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "gray", size: "sm" }}
                                        className="w-full justify-start text-gray-200 hover:bg-[#3d3d3d]"
                                        leftIcon={<Globe className="w-4 h-4" />}
                                        onClick={onNewPortal}
                                    >
                                        New Portal
                                    </Button>
                                )}
                            </Menu.Item>

                            <div className="border-t border-[#444] my-1" />

                            <Menu.Item>
                                {() => (
                                    <Tooltip
                                        label={!hasPortal ? PORTAL_REQUIRED_TOOLTIP : ""}
                                        isDisabled={hasPortal}
                                        placement="right"
                                        hasArrow
                                    >
                                        <div>
                                            <Button
                                                buttonVariant="ghost"
                                                buttonStyle={{ color: "gray", size: "sm" }}
                                                className={`w-full justify-start ${hasPortal ? "text-gray-200 hover:bg-[#3d3d3d]" : "text-gray-600 cursor-not-allowed"}`}
                                                leftIcon={<Terminal className={`w-4 h-4 ${hasPortal ? "" : "opacity-40"}`} />}
                                                onClick={() => setSshModalOpen(true)}
                                                disabled={!hasPortal}
                                            >
                                                SSH
                                            </Button>
                                        </div>
                                    </Tooltip>
                                )}
                            </Menu.Item>

                            <Menu.Item>
                                {() => (
                                    <Tooltip
                                        label={!hasPortal ? PORTAL_REQUIRED_TOOLTIP : ""}
                                        isDisabled={hasPortal}
                                        placement="right"
                                        hasArrow
                                    >
                                        <div>
                                            <Button
                                                buttonVariant="ghost"
                                                buttonStyle={{ color: "gray", size: "sm" }}
                                                className={`w-full justify-start ${hasPortal ? "text-gray-200 hover:bg-[#3d3d3d]" : "text-gray-600 cursor-not-allowed"}`}
                                                leftIcon={<SquareTerminal className={`w-4 h-4 ${hasPortal ? "" : "opacity-40"}`} />}
                                                onClick={onPtyOpen}
                                                disabled={!hasPortal}
                                            >
                                                PTY
                                            </Button>
                                        </div>
                                    </Tooltip>
                                )}
                            </Menu.Item>
                        </div>
                    </Menu.Items>
                </Transition>
            </Menu>

            <SshConnectionModal
                isOpen={sshModalOpen}
                onClose={() => setSshModalOpen(false)}
                onConnect={onSshConnect}
            />
        </>
    );
};

export default ShellActionsMenu;
