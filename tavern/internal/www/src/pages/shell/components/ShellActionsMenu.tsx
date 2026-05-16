import React, { Fragment, useState, useEffect } from "react";
import { Menu, Transition } from "@headlessui/react";
import { Share, Terminal, Globe, SquareTerminal, Ellipsis, X, StopCircle, Search } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import SshConnectionModal from "./SshConnectionModal";

interface ShellActionsMenuProps {
    portalId: number | null;
    onExport: () => void;
    onNewPortal: () => void;
    onClosePortal: () => void;
    onSshConnect: (target: string) => void;
    onPtyOpen: () => void;
    onSendCtrlC: () => void;
    onSendCtrlR: () => void;
}

const PORTAL_REQUIRED_TOOLTIP = "Requires an active portal connection. Create a portal first using pivot.create_portal().";

const ShellActionsMenu: React.FC<ShellActionsMenuProps> = ({
    portalId,
    onExport,
    onNewPortal,
    onClosePortal,
    onSshConnect,
    onPtyOpen,
    onSendCtrlC,
    onSendCtrlR,
}) => {
    const [sshModalOpen, setSshModalOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(false);
    const hasPortal = portalId !== null;

    useEffect(() => {
        const checkMobile = () => {
            setIsMobile(window.innerWidth < 768);
        };
        checkMobile();
        window.addEventListener("resize", checkMobile);
        return () => window.removeEventListener("resize", checkMobile);
    }, []);

    return (
        <>
            <Menu as="div" className="relative">
                <Menu.Button
                    className="p-1 rounded text-white hover:bg-[#3d3d3d] transition-colors focus:outline-none"
                    aria-label="Shell actions"
                >
                    <Ellipsis className="w-5 h-5" />
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
                    <Menu.Items className="absolute right-0 mt-2 w-56 origin-top-right rounded-md bg-[#2d2d2d] border border-[#444] shadow-lg focus:outline-none z-50">
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

                            {isMobile && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#3d3d3d]"
                                            leftIcon={<StopCircle className="w-4 h-4" />}
                                            onClick={onSendCtrlC}
                                        >
                                            Send Ctrl + C
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            {isMobile && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#3d3d3d]"
                                            leftIcon={<Search className="w-4 h-4" />}
                                            onClick={onSendCtrlR}
                                        >
                                            Send Ctrl + R
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            <Menu.Item>
                                {() => (
                                    hasPortal ? (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-red-400 hover:bg-[#3d3d3d]"
                                            leftIcon={<X className="w-4 h-4" />}
                                            onClick={onClosePortal}
                                        >
                                            Close Portal
                                        </Button>
                                    ) : (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#3d3d3d]"
                                            leftIcon={<Globe className="w-4 h-4" />}
                                            onClick={onNewPortal}
                                        >
                                            New Portal
                                        </Button>
                                    )
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
