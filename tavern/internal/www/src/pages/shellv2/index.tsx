import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import { Tabs, TabList, TabPanels, Tab, TabPanel } from '@chakra-ui/react';

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
    } = useShellTerminal(shellId, loading, error, shellData, setPortalId, isLateCheckin);

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

    // TODO: This should be updated when we begin to open new tabs for pivot connections (e.g. SSH)
    let useTabs = false;

    let shellTerm = <ShellTerminal
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
    />;
    if (useTabs) {
        shellTerm = <Tabs variant="enclosed" flex="1" display="flex" flexDirection="column" mt={4} overflow="hidden">
            <TabList borderBottomColor="#333">
                <Tab _selected={{ color: 'white', bg: '#2d2d2d', borderColor: '#333', borderBottomColor: 'transparent' }} color="#888" borderColor="transparent">{shellData?.node?.beacon?.name ?? "Shell"}</Tab>
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
            </TabPanels>
        </Tabs>;
    }

    return (
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader shellData={shellData} activeUsers={activeUsers} />

                {shellTerm}

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
