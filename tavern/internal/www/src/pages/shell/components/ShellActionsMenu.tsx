import React, { Fragment, useState, useEffect } from "react";
import { Menu, Transition } from "@headlessui/react";
import { BookOpen, Compass, Share, Ellipsis, StopCircle, Search, RefreshCw, Bug } from "lucide-react";
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
    onReplayTutorial?: () => void;
    sshModalOpenExternal?: boolean;
    sshModalInitialTarget?: string;
    onCloseSshModalExternal?: () => void;
    credentials?: KnowledgeCredentialItem[];
    hosts?: KnowledgeHostItem[];
    onLoadIntoComposer?: (code: string) => void;
    buttonClassName?: string;
    forceOpen?: boolean;
}

const ShellActionsMenu: React.FC<ShellActionsMenuProps> = ({
    onExport,
    onSshConnect,
    onSendCtrlC,
    onSendCtrlR,
    onSpawnFreshShell,
    onReplayTutorial,
    sshModalOpenExternal,
    sshModalInitialTarget,
    onCloseSshModalExternal,
    credentials,
    hosts,
    onLoadIntoComposer,
    buttonClassName,
    forceOpen = false,
}) => {
    const [sshModalOpenInternal, setSshModalOpenInternal] = useState(false);
    const [isMobile, setIsMobile] = useState(false);

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
                {({ open }) => {
                    const isMenuOpen = open || Boolean(forceOpen);
                    return (
                        <>
                            <Menu.Button
                                data-tour="shell-actions-button"
                                className={
                                    buttonClassName ||
                                    "relative inline-flex items-center justify-center w-7 h-7 rounded-md transition-colors bg-transparent hover:bg-[#282f42] text-gray-300 hover:text-white focus:outline-none"
                                }
                                aria-label="Shell actions"
                            >
                                <Ellipsis className="w-[15px] h-[15px]" />
                            </Menu.Button>
                            <Transition
                                show={isMenuOpen}
                                as={Fragment}
                                enter={forceOpen ? "" : "transition ease-out duration-100"}
                                enterFrom={forceOpen ? "" : "transform opacity-0 scale-95"}
                                enterTo={forceOpen ? "" : "transform opacity-100 scale-100"}
                                leave="transition ease-in duration-75"
                                leaveFrom="transform opacity-100 scale-100"
                                leaveTo="transform opacity-0 scale-95"
                            >
                                <Menu.Items
                                    static
                                    className="absolute right-0 mt-2 w-60 origin-top-right rounded-md bg-[#1f2430] border border-[#343b4d] shadow-xl focus:outline-none z-50"
                                >
                                    <div className="px-1.5 py-1.5 space-y-0.5">
                                        <Menu.Item>
                                            {() => (
                                                <Button
                                                    buttonVariant="ghost"
                                                    buttonStyle={{ color: "gray", size: "sm" }}
                                                    className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                    leftIcon={<Share className="w-4 h-4" />}
                                                    onClick={onExport}
                                                >
                                                    Export
                                                </Button>
                                            )}
                                        </Menu.Item>

                                        {onSpawnFreshShell && (
                                            <Menu.Item>
                                                {() => (
                                                    <Button
                                                        buttonVariant="ghost"
                                                        buttonStyle={{ color: "gray", size: "sm" }}
                                                        className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                        leftIcon={<RefreshCw className="w-4 h-4" />}
                                                        onClick={onSpawnFreshShell}
                                                    >
                                                        New Interpreter
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
                                                        className="w-full justify-start text-white hover:bg-[#2b3242]"
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
                                                        className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                        leftIcon={<Search className="w-4 h-4" />}
                                                        onClick={onSendCtrlR}
                                                    >
                                                        Send Ctrl + R
                                                    </Button>
                                                )}
                                            </Menu.Item>
                                        )}

                                        <div className="border-t border-[#343b4d] my-1" />

                                        <div data-tour="shell-guide" className="space-y-0.5">
                                            {onReplayTutorial && (
                                                <Menu.Item>
                                                    {() => (
                                                        <Button
                                                            buttonVariant="ghost"
                                                            buttonStyle={{ color: "gray", size: "sm" }}
                                                            className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                            leftIcon={<Compass className="w-4 h-4" />}
                                                            onClick={onReplayTutorial}
                                                        >
                                                            Replay Tutorial
                                                        </Button>
                                                    )}
                                                </Menu.Item>
                                            )}

                                            <Menu.Item>
                                                {() => (
                                                    <a
                                                        href="https://docs.realm.pub/user-guide/eldritch"
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        aria-label="Documentation"
                                                        className="block w-full"
                                                    >
                                                        <Button
                                                            buttonVariant="ghost"
                                                            buttonStyle={{ color: "gray", size: "sm" }}
                                                            className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                            leftIcon={<BookOpen className="w-4 h-4" />}
                                                        >
                                                            Documentation
                                                        </Button>
                                                    </a>
                                                )}
                                            </Menu.Item>
                                        </div>

                                        <Menu.Item>
                                            {() => (
                                                <a
                                                    data-tour="shell-bug-report"
                                                    href="https://github.com/spellshift/realm/issues/new?template=bug_report.md"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    aria-label="Report a bug"
                                                    className="block w-full"
                                                >
                                                    <Button
                                                        buttonVariant="ghost"
                                                        buttonStyle={{ color: "gray", size: "sm" }}
                                                        className="w-full justify-start text-white hover:bg-[#2b3242]"
                                                        leftIcon={<Bug className="w-4 h-4" />}
                                                    >
                                                        Report a Bug
                                                    </Button>
                                                </a>
                                            )}
                                        </Menu.Item>
                                    </div>
                                </Menu.Items>
                            </Transition>
                        </>
                    );
                }}
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
