import { useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import { useState, useEffect, useRef } from "react";
import { Tabs, TabList, TabPanels, Tab, TabPanel, IconButton } from "@chakra-ui/react";
import { CloseIcon } from "@chakra-ui/icons";
import SshTerminal from "./components/SshTerminal";

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

    const [sshTabs, setSshTabs] = useState<{ id: string, portalId: number, title: string, initialCommand?: string }[]>([]);
    const [tabIndex, setTabIndex] = useState(0);

    const [pendingSshRequests, setPendingSshRequests] = useState<{ host: string, id: string }[]>([]);

    useEffect(() => {
        const handleMeta = (e: Event) => {
            const detail = (e as CustomEvent).detail;
            if (detail.command === "ssh") {
                setPendingSshRequests(prev => [...prev, { host: detail.args[0] || "Unknown Host", id: crypto.randomUUID() }]);
            }
        };
        window.addEventListener("ELD_META_COMMAND", handleMeta);
        return () => window.removeEventListener("ELD_META_COMMAND", handleMeta);
    }, []);

    useEffect(() => {
        if (!portalId) return;

        let addedTabs = 0;

        if (pendingSshRequests.length > 0) {
            setSshTabs(prev => {
                const newTabs = pendingSshRequests.map(req => ({
                    id: req.id,
                    portalId,
                    title: `SSH: ${req.host}`,
                    initialCommand: `ssh ${req.host}\r`
                }));
                addedTabs += newTabs.length;
                return [...prev, ...newTabs];
            });
            setPendingSshRequests([]);
        }

        if (addedTabs > 0) {
            setTabIndex(prev => prev + addedTabs);
        }
    }, [portalId, pendingSshRequests]);

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

    const handleCloseTab = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setSshTabs(prev => prev.filter(t => t.id !== id));
        setTabIndex(0);
    };

    return (
        <AccessGate>
            <div className="flex flex-col h-screen p-5 bg-[#1e1e1e] text-[#d4d4d4]">
                <ShellHeader shellData={shellData} activeUsers={activeUsers} />

                <Tabs
                    index={tabIndex}
                    onChange={setTabIndex}
                    variant="enclosed"
                    colorScheme="purple"
                    size="sm"
                    className="flex-1 flex flex-col min-h-0 mt-4"
                >
                    <TabList borderBottom="1px solid #333">
                        <Tab _selected={{ color: "white", bg: "#2d2d2d", borderColor: "#333", borderBottomColor: "#2d2d2d" }} color="gray.400" border="1px solid transparent">
                            Main Shell
                        </Tab>
                        {sshTabs.map((tab) => (
                            <Tab key={tab.id} _selected={{ color: "white", bg: "#2d2d2d", borderColor: "#333", borderBottomColor: "#2d2d2d" }} color="gray.400" border="1px solid transparent">
                                {tab.title}
                                <IconButton
                                    aria-label="Close tab"
                                    icon={<CloseIcon />}
                                    size="xs"
                                    ml={2}
                                    variant="ghost"
                                    _hover={{ bg: "red.500", color: "white" }}
                                    onClick={(e) => handleCloseTab(tab.id, e)}
                                />
                            </Tab>
                        ))}
                    </TabList>

                    <TabPanels className="flex-1 min-h-0 relative">
                        <TabPanel p={0} h="100%" className="absolute inset-0">
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
                        {sshTabs.map(tab => (
                            <TabPanel key={tab.id} p={0} h="100%" className="absolute inset-0 pt-2">
                                <SshTerminal portalId={tab.portalId} initialCommand={tab.initialCommand} />
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
