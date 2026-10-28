import React, { Fragment, useState, useEffect } from "react";
import { Menu, Transition } from "@headlessui/react";
import { Share, Terminal, Globe, SquareTerminal, Ellipsis, X, StopCircle, Search, RefreshCw, History, BookOpen, Code2, Database } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import Button from "../../../components/tavern-base-ui/button/Button";
import SshConnectionModal from "./SshConnectionModal";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";

interface ShellActionsMenuProps {
    portalId: number | null;
    onExport: () => void;
    onNewPortal: () => void;
    onClosePortal: () => void;
    onSshConnect: (target: string) => void;
    onPtyOpen: () => void;
    onSendCtrlC: () => void;
    onSendCtrlR: () => void;
    onSpawnFreshShell?: () => void;
    onToggleHistory?: () => void;
    onToggleKnowledge?: () => void;
    onToggleNux?: () => void;
    onToggleComposer?: () => void;
    sshModalOpenExternal?: boolean;
    sshModalInitialTarget?: string;
    onCloseSshModalExternal?: () => void;
    credentials?: KnowledgeCredentialItem[];
    hosts?: KnowledgeHostItem[];
    onLoadIntoComposer?: (code: string) => void;
}

const PORTAL_REQUIRED_TOOLTIP = "Requires an active portal connection. Create a portal first using portal.create().";

const ShellActionsMenu: React.FC<ShellActionsMenuProps> = ({
    portalId,
    onExport,
    onNewPortal,
    onClosePortal,
    onSshConnect,
    onPtyOpen,
    onSendCtrlC,
    onSendCtrlR,
    onSpawnFreshShell,
    onToggleHistory,
    onToggleKnowledge,
    onToggleNux,
    onToggleComposer,
    sshModalOpenExternal,
    sshModalInitialTarget,
    onCloseSshModalExternal,
    credentials,
    hosts,
    onLoadIntoComposer,
}) => {
    const [sshModalOpenInternal, setSshModalOpenInternal] = useState(false);
    const [isMobile, setIsMobile] = useState(false);
    const hasPortal = portalId !== null;

    const isSshModalOpen = sshModalOpenExternal !== undefined ? (sshModalOpenExternal || sshModalOpenInternal) : sshModalOpenInternal;
    const closeSshModal = () => {
        setSshModalOpenInternal(false);
        onCloseSshModalExternal?.();
    };

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
                    className="p-1.5 rounded text-gray-200 hover:text-white hover:bg-[#2a2f3d] border border-transparent hover:border-[#3a4154] transition-colors focus:outline-none"
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
                    <Menu.Items className="absolute right-0 mt-2 w-60 origin-top-right rounded-md bg-[#1f2430] border border-[#343b4d] shadow-xl focus:outline-none z-50">
                        <div className="px-1.5 py-1.5 space-y-0.5">
                            <Menu.Item>
                                {() => (
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "gray", size: "sm" }}
                                        className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                        leftIcon={<Share className="w-4 h-4" />}
                                        onClick={onExport}
                                    >
                                        Export
                                    </Button>
                                )}
                            </Menu.Item>

                            {onToggleKnowledge && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                            leftIcon={<Database className="w-4 h-4" />}
                                            onClick={onToggleKnowledge}
                                        >
                                            Knowledge Graph
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            {onToggleHistory && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                            leftIcon={<History className="w-4 h-4" />}
                                            onClick={onToggleHistory}
                                        >
                                            History &amp; Task Queue
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            {onToggleComposer && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                            leftIcon={<Code2 className="w-4 h-4" />}
                                            onClick={onToggleComposer}
                                        >
                                            Composer
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            {onToggleNux && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                            leftIcon={<BookOpen className="w-4 h-4" />}
                                            onClick={onToggleNux}
                                        >
                                            Operator Guide &amp; Docs
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            {onSpawnFreshShell && (
                                <Menu.Item>
                                    {() => (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-amber-300 hover:bg-[#2b3242]"
                                            leftIcon={<RefreshCw className="w-4 h-4" />}
                                            onClick={onSpawnFreshShell}
                                        >
                                            Spawn new shell
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
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
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
                                            className="w-full justify-start text-gray-200 hover:bg-[#2b3242]"
                                            leftIcon={<Search className="w-4 h-4" />}
                                            onClick={onSendCtrlR}
                                        >
                                            Send Ctrl + R
                                        </Button>
                                    )}
                                </Menu.Item>
                            )}

                            <div className="border-t border-[#343b4d] my-1" />

                            <Menu.Item>
                                {() => (
                                    hasPortal ? (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-red-400 hover:bg-[#2b3242]"
                                            leftIcon={<X className="w-4 h-4" />}
                                            onClick={onClosePortal}
                                        >
                                            Close Portal
                                        </Button>
                                    ) : (
                                        <Button
                                            buttonVariant="ghost"
                                            buttonStyle={{ color: "gray", size: "sm" }}
                                            className="w-full justify-start text-emerald-300 hover:bg-[#2b3242]"
                                            leftIcon={<Globe className="w-4 h-4" />}
                                            onClick={onNewPortal}
                                        >
                                            New Portal
                                        </Button>
                                    )
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
                                                className={`w-full justify-start ${hasPortal ? "text-gray-200 hover:bg-[#2b3242]" : "text-gray-600 cursor-not-allowed"}`}
                                                leftIcon={<Terminal className={`w-4 h-4 ${hasPortal ? "" : "opacity-40"}`} />}
                                                onClick={() => setSshModalOpenInternal(true)}
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
                                                className={`w-full justify-start ${hasPortal ? "text-gray-200 hover:bg-[#2b3242]" : "text-gray-600 cursor-not-allowed"}`}
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
                isOpen={isSshModalOpen}
                onClose={closeSshModal}
                onConnect={onSshConnect}
                initialTarget={sshModalInitialTarget}
                credentials={credentials}
                hosts={hosts}
                onLoadIntoComposer={onLoadIntoComposer}
            />
        </>
    );
};

export default ShellActionsMenu;
