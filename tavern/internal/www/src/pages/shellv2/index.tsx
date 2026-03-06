import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import { AccessGate } from "../../components/access-gate";
import { Tabs, TabList, TabPanels, Tab, TabPanel, Box } from "@chakra-ui/react";
import { TerminalIcon, XIcon } from "lucide-react";
import { ShellSession } from "./components/ShellSession";
import { ShellSSH } from "./components/ShellSSH";
import { useShellData } from "./hooks/useShellData";

interface SshSession {
    id: string;
    target: string;
    portalId: number;
}

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();
    const { shellData } = useShellData(shellId);
    const beaconName = shellData?.node?.beacon?.name || "Shell";

    const [sshSessions, setSshSessions] = useState<SshSession[]>([]);
    const [selectedIndex, setSelectedIndex] = useState(0);

    const handleSshConnect = useCallback((target: string, portalId: number) => {
        const newSession = {
            id: Math.random().toString(36).substring(7),
            target,
            portalId
        };
        setSshSessions(prev => {
            setSelectedIndex(prev.length + 1);
            return [...prev, newSession];
        });
    }, []);

    const handleCloseSsh = (e: React.MouseEvent, id: string) => {
        e.stopPropagation();
        setSshSessions(prev => {
            const index = prev.findIndex(s => s.id === id);
            if (index !== -1 && selectedIndex > index) {
                setSelectedIndex(selectedIndex - 1);
            } else if (selectedIndex > prev.length - 1) {
                setSelectedIndex(prev.length - 1);
            }
            return prev.filter(s => s.id !== id);
        });
    };

    // For testing/verification script support
    useEffect(() => {
        const handler = (e: Event) => {
            const customEvent = e as CustomEvent;
            if (customEvent.detail && customEvent.detail.target && customEvent.detail.portalId) {
                handleSshConnect(customEvent.detail.target, customEvent.detail.portalId);
            }
        };
        window.addEventListener('simulateSshConnect', handler);
        return () => window.removeEventListener('simulateSshConnect', handler);
    }, [handleSshConnect]);

    return (
        <AccessGate>
            <div className="flex flex-col h-screen bg-[#1e1e1e]">
                <Tabs index={selectedIndex} onChange={setSelectedIndex} className="flex flex-col h-full" isLazy={false}>
                    {sshSessions.length > 0 && (
                        <TabList className="flex flex-row space-x-1 border-b border-gray-700 bg-[#2d2d2d] pt-2 px-2" borderBottom="none">
                            <Tab
                                className={`px-4 py-2 flex flex-row gap-2 items-center rounded-t-lg outline-none ${selectedIndex === 0 ? 'bg-[#1e1e1e] text-[#d4d4d4]' : 'bg-[#3d3d3d] text-gray-400 hover:bg-[#4d4d4d]'}`}
                                _selected={{ bg: '#1e1e1e', color: '#d4d4d4', border: 'none' }}
                                border="none"
                            >
                                <TerminalIcon className="w-4 h-4" />
                                <div>{beaconName}</div>
                            </Tab>
                            {sshSessions.map((session, i) => (
                                <Tab
                                    key={session.id}
                                    className={`px-4 py-2 flex flex-row gap-2 items-center rounded-t-lg outline-none group ${selectedIndex === i + 1 ? 'bg-[#1e1e1e] text-[#d4d4d4]' : 'bg-[#3d3d3d] text-gray-400 hover:bg-[#4d4d4d]'}`}
                                    _selected={{ bg: '#1e1e1e', color: '#d4d4d4', border: 'none' }}
                                    border="none"
                                >
                                    <TerminalIcon className="w-4 h-4" />
                                    <div>{session.target} (SSH)</div>
                                    <button
                                        onClick={(e) => handleCloseSsh(e, session.id)}
                                        className="ml-2 p-0.5 rounded-sm hover:bg-gray-600 opacity-0 group-hover:opacity-100 transition-opacity"
                                    >
                                        <XIcon className="w-3 h-3" />
                                    </button>
                                </Tab>
                            ))}
                        </TabList>
                    )}
                    <TabPanels className="flex-1 overflow-hidden" h="full">
                        <TabPanel className="h-full p-0 m-0 outline-none" h="full" p={0}>
                            <ShellSession shellId={shellId} onSshConnect={handleSshConnect} />
                        </TabPanel>
                        {sshSessions.map(session => (
                            <TabPanel key={session.id} className="h-full p-0 m-0 outline-none" h="full" p={0}>
                                <ShellSSH target={session.target} portalId={session.portalId} />
                            </TabPanel>
                        ))}
                    </TabPanels>
                </Tabs>
            </div>
        </AccessGate>
    );
};

export default ShellV2;
