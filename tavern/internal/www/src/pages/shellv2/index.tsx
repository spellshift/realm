import { useState } from "react";
import { useParams } from "react-router-dom";
import { Tabs, TabList, Tab, TabPanels, TabPanel } from "@chakra-ui/react";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import { v4 as uuidv4 } from "uuid";
import SSHTerminal from "./components/SSHTerminal";

const ShellV2 = () => {
    const { shellId } = useParams<{ shellId: string }>();

    const [sshTabs, setSshTabs] = useState<{ id: string; target: string; sessionId: string }[]>([]);

    const handleMetaSSH = (target: string) => {
        const sessionId = uuidv4();
        setSshTabs((prev) => [...prev, { id: uuidv4(), target, sessionId }]);
    };

    const handleCloseSshTab = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setSshTabs((prev) => prev.filter((tab) => tab.id !== id));
    };

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
    } = useShellTerminal(shellId, loading, error, shellData, portalId, setPortalId, isLateCheckin, handleMetaSSH);

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

                <Tabs isLazy={false} display="flex" flexDirection="column" flex="1" overflow="hidden" variant="enclosed" colorScheme="blue">
                    <TabList mb="1em">
                        <Tab>Shell</Tab>
                        {sshTabs.map((tab) => (
                            <Tab key={tab.id}>
                                {tab.target}
                                <button
                                    className="ml-2 text-gray-500 hover:text-red-500"
                                    onClick={(e) => handleCloseSshTab(tab.id, e)}
                                >
                                    &times;
                                </button>
                            </Tab>
                        ))}
                    </TabList>
                    <TabPanels flex="1" overflow="hidden" display="flex" flexDirection="column">
                        <TabPanel p={0} flex="1" display="flex" flexDirection="column" overflow="hidden">
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
                        {sshTabs.map((tab) => (
                            <TabPanel key={tab.id} p={0} flex="1" display="flex" flexDirection="column" overflow="hidden">
                                {portalId !== null && (
                                    <SSHTerminal portalId={portalId} sessionId={tab.sessionId} target={tab.target} />
                                )}
                            </TabPanel>
                        ))}
                    </TabPanels>
                </Tabs>

                <ShellStatusBar
                    portalId={portalId}
                    timeUntilCallback={timeUntilCallback}
                    isMissedCallback={isMissedCallback}
                    connectionStatus={connectionStatus}
                    connectionMessage={connectionMessage}
                />
            </div>
        </AccessGate>
    );
};

export default ShellV2;
