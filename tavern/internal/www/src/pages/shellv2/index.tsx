import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import { Tabs, TabList, TabPanels, Tab, TabPanel, useToast, Tooltip } from '@chakra-ui/react';
import { useState, useEffect, useCallback } from 'react';
import { useMutation } from '@apollo/client';
import SshTerminal from './components/SshTerminal';
import PtyTerminal from './components/PtyTerminal';
import { CLOSE_PORTAL_MUTATION } from './graphql';
import { WifiOff } from 'lucide-react';

interface PortalTab {
    id: string;
    type: string;
    target: string;
    pivotId?: string;
}

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const {
        loading,
        error,
        shellData,
        beaconData,
        portalId,
        setPortalId,
        activeUsers
    } = useShellData(shellId);

    const { timeUntilCallback, isMissedCallback, isLateCheckin } = useCallbackTimer(beaconData);

    const [portalTabs, setPortalTabs] = useState<PortalTab[]>([]);
    const [tabIndex, setTabIndex] = useState(0);
    const [disconnectedTabs, setDisconnectedTabs] = useState<Set<string>>(new Set());

    const handleTabConnectionStatusChange = useCallback((tabId: string, status: "connecting" | "connected" | "disconnected") => {
        setDisconnectedTabs(prev => {
            const next = new Set(prev);
            if (status === "disconnected") {
                next.add(tabId);
            } else {
                next.delete(tabId);
            }
            return next;
        });
    }, []);

    useEffect(() => {
        if (shellData?.node?.pivots?.edges) {
            shellData.node.pivots.edges.forEach((edge: any) => {
                const pivot = edge.node;
                if (!pivot.closedAt) {
                    handleOpenPortalTab(pivot.kind, pivot.destination, pivot.id);
                }
            });
        }
    }, [shellData]);

    const handleOpenPortalTab = (type: string, target: string, pivotId?: string) => {
        const id = pivotId ? `pivot-${pivotId}` : `${type}-${target}-${Date.now()}`;
        setPortalTabs(prev => {
            if (prev.find(t => t.id === id)) return prev;
            const newTabs = [...prev, { id, type, target, pivotId }];
            setTabIndex(newTabs.length); // index 0 is main shell, so new length is the correct index
            return newTabs;
        });
    };

    const {
        termRef,
        connectionError,
        completions,
        showCompletions,
        completionPos,
        completionIndex,
        handleMouseMove,
        tooltipState,
        handleCompletionSelect,
        connectionStatus,
        connectionMessage,
        handleTooltipMouseEnter,
        handleTooltipMouseLeave,
        getSessionInputs,
        setShellInput,
        focusTerminal
    } = useShellTerminal(shellId, loading, error, shellData, setPortalId, isLateCheckin, handleOpenPortalTab);

    const toast = useToast();

    const [closePortalMutation] = useMutation(CLOSE_PORTAL_MUTATION);

    const handleExport = () => {
        const inputs = getSessionInputs();
        if (inputs) {
            navigator.clipboard.writeText(inputs);
            toast({
                title: "Copied to clipboard",
                description: "Session input history has been copied to your clipboard.",
                status: "success",
                duration: 3000,
                isClosable: true,
            });
        } else {
            toast({
                title: "No inputs to export",
                description: "There are no completed commands in the current session.",
                status: "info",
                duration: 3000,
                isClosable: true,
            });
        }
    };

    const handleNewPortal = () => {
        setShellInput("pivot.create_portal()");
        // HeadlessUI Menu restores focus to Menu.Button after onClick;
        // delay refocus so the terminal regains focus after the menu closes.
        setTimeout(() => focusTerminal(), 0);
    };

    const handleClosePortal = async () => {
        if (!portalId) return;
        try {
            await closePortalMutation({ variables: { id: portalId } });
            setPortalId(null);
            toast({
                title: "Portal closed",
                description: "The active portal has been closed.",
                status: "success",
                duration: 3000,
                isClosable: true,
            });
        } catch (err: any) {
            toast({
                title: "Failed to close portal",
                description: err?.message || "An error occurred while closing the portal.",
                status: "error",
                duration: 5000,
                isClosable: true,
            });
        }
    };

    const handleSshConnect = (target: string) => {
        handleOpenPortalTab("ssh", target);
    };

    const handlePtyOpen = () => {
        handleOpenPortalTab("pty", "PTY");
    };

    if (connectionError) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="Shell Connection Failed" details={connectionError} />
            </div>
        );
    }

    if (loading) {
        return <div style={{ padding: "20px", color: "#d4d4d4" }}>Loading Shell...</div>;
    }

    const useTabs = portalTabs.length > 0;

    let shellTerm = (
        <Tabs index={tabIndex} onChange={(index) => setTabIndex(index)} variant="enclosed" flex="1" display="flex" flexDirection="column" mt={useTabs ? 4 : 0} overflow="hidden">
            <TabList borderBottomColor="#333" display={useTabs ? 'flex' : 'none'}>
                <Tab _selected={{ color: 'white', bg: '#2d2d2d', borderColor: '#333', borderBottomColor: 'transparent' }} color="#888" borderColor="transparent">{shellData?.node?.beacon?.name ?? "Shell"}</Tab>
                {portalTabs.map(tab => (
                    <Tab key={tab.id} _selected={{ color: 'white', bg: '#2d2d2d', borderColor: '#333', borderBottomColor: 'transparent' }} color="#888" borderColor="transparent">
                        <span className="flex items-center gap-1.5">
                            {disconnectedTabs.has(tab.id) && (
                                <Tooltip label="Disconnected" hasArrow>
                                    <span className="text-yellow-500"><WifiOff size={14} /></span>
                                </Tooltip>
                            )}
                            {tab.target}
                        </span>
                    </Tab>
                ))}
            </TabList>
            <TabPanels flex="1" display="flex" flexDirection="column" overflow="hidden">
                <TabPanel flex="1" p={0} display="flex" flexDirection="column" overflow="hidden">
                    <ShellTerminal
                        termRef={termRef}
                        completions={completions}
                        showCompletions={showCompletions}
                        completionPos={completionPos}
                        completionIndex={completionIndex}
                        onMouseMove={handleMouseMove}
                        onMouseLeave={handleTooltipMouseLeave}
                        tooltipState={tooltipState}
                        handleCompletionSelect={handleCompletionSelect}
                        onTooltipMouseEnter={handleTooltipMouseEnter}
                        onTooltipMouseLeave={handleTooltipMouseLeave}
                    />
                </TabPanel>
                {portalTabs.map(tab => (
                    <TabPanel key={tab.id} flex="1" p={0} display="flex" flexDirection="column" overflow="hidden">
                        {tab.type === "ssh" && (portalId || tab.pivotId) && (
                            <SshTerminal portalId={portalId || 0} target={tab.target} pivotId={tab.pivotId ? parseInt(tab.pivotId) : undefined} shellId={shellId || ""} onConnectionStatusChange={(status) => handleTabConnectionStatusChange(tab.id, status)} />
                        )}
                        {tab.type === "pty" && (portalId || tab.pivotId) && (
                            <PtyTerminal portalId={portalId || 0} pivotId={tab.pivotId ? parseInt(tab.pivotId) : undefined} shellId={shellId || ""} onConnectionStatusChange={(status) => handleTabConnectionStatusChange(tab.id, status)} />
                        )}
                    </TabPanel>
                ))}
            </TabPanels>
        </Tabs>
    );

    return (
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader
                    shellData={shellData}
                    activeUsers={activeUsers}
                    portalId={portalId}
                    onExport={handleExport}
                    onNewPortal={handleNewPortal}
                    onClosePortal={handleClosePortal}
                    onSshConnect={handleSshConnect}
                    onPtyOpen={handlePtyOpen}
                />

                {shellTerm}

                <ShellStatusBar
                    portalId={portalId}
                    timeUntilCallback={timeUntilCallback}
                    isMissedCallback={isMissedCallback}
                    connectionStatus={connectionStatus}
                    connectionMessage={connectionMessage}
                    closedAt={shellData?.node?.closedAt}
                />
            </div>
        </AccessGate>
    );
};

export default ShellV2;
