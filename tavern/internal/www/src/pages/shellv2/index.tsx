import React, { useState } from "react";
import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import SshTerminal from "./components/SshTerminal";
import { Tabs, TabList, TabPanels, Tab, TabPanel, CloseButton } from "@chakra-ui/react";

interface TabData {
    id: string;
    type: "shell" | "ssh";
    title: string;
    target?: string;
    sessionId?: string;
}

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const {
        loading,
        error,
        shellData,
        beaconData,
        portalData,
        portalId,
        setPortalId,
        activeUsers
    } = useShellData(shellId);

    const { timeUntilCallback, isMissedCallback, isLateCheckin } = useCallbackTimer(beaconData);

    const [tabs, setTabs] = useState<TabData[]>([
        { id: "main-shell", type: "shell", title: "Shell" }
    ]);
    const [activeTabIndex, setActiveTabIndex] = useState(0);

        const portalIdRef = React.useRef(portalId);
    React.useEffect(() => { portalIdRef.current = portalId; }, [portalId]);

    const handleMetaSsh = React.useCallback((target: string) => {
        if (!portalIdRef.current) {
            return "No active portal connection. Wait for connection or restart shell.";
        }

        const newSessionId = crypto.randomUUID();
        const newTabId = `ssh-${newSessionId}`;
        setTabs(prev => [
            ...prev,
            { id: newTabId, type: "ssh", title: `SSH: ${target}`, target, sessionId: newSessionId }
        ]);
        setActiveTabIndex(tabs.length);
        return null; // Null means success
    }, [tabs.length]);

    const handleCloseTab = (index: number) => {
        const newTabs = tabs.filter((_, i) => i !== index);
        setTabs(newTabs);
        if (activeTabIndex >= index && activeTabIndex > 0) {
            setActiveTabIndex(activeTabIndex - 1);
        } else if (activeTabIndex === newTabs.length) {
            setActiveTabIndex(newTabs.length - 1);
        }
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
        handleTooltipMouseLeave
    } = useShellTerminal(shellId, loading, error, shellData, setPortalId, isLateCheckin, handleMetaSsh);

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

    return (
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader shellData={shellData} activeUsers={activeUsers} />

                <Tabs isLazy={false} index={activeTabIndex} onChange={setActiveTabIndex} variant="enclosed" colorScheme="blue" flex="1" display="flex" flexDirection="column" overflow="hidden">
                    <TabList mb="1em">
                        {tabs.map((tab, idx) => (
                            <Tab key={tab.id} _selected={{ color: 'white', bg: '#333' }} color="gray.400">
                                {tab.title}
                                {tab.type !== "shell" && (
                                    <CloseButton size="sm" ml={2} onClick={(e) => { e.stopPropagation(); handleCloseTab(idx); }} />
                                )}
                            </Tab>
                        ))}
                    </TabList>
                    <TabPanels flex="1" display="flex" overflow="hidden">
                        {tabs.map((tab) => (
                            <TabPanel key={tab.id} p={0} flex="1" display="flex" flexDirection="column" height="100%">
                                {tab.type === "shell" ? (
                                    <>
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
                                        <ShellStatusBar
                                            portalId={portalId}
                                            timeUntilCallback={timeUntilCallback}
                                            isMissedCallback={isMissedCallback}
                                            connectionStatus={connectionStatus}
                                            connectionMessage={connectionMessage}
                                        />
                                    </>
                                ) : (
                                    <SshTerminal portalId={portalId!} sessionId={tab.sessionId!} target={tab.target!} />
                                )}
                            </TabPanel>
                        ))}
                    </TabPanels>
                </Tabs>
            </div>
        </AccessGate>
    );
};

export default ShellV2;
