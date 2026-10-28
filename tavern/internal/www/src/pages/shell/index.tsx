import { useNavigate, useParams } from "react-router-dom";
import AlertError from "../../components/tavern-base-ui/AlertError";
import { AccessGate } from "../../components/access-gate";
import { useShellData } from "./hooks/useShellData";
import { useCallbackTimer } from "./hooks/useCallbackTimer";
import { useShellTerminal } from "./hooks/useShellTerminal";
import { useTabHotkeys } from "./hooks/useTabHotkeys";
import ShellHeader from "./components/ShellHeader";
import ShellTerminal from "./components/ShellTerminal";
import ShellStatusBar from "./components/ShellStatusBar";
import InterpreterStatusBanner from "./components/InterpreterStatusBanner";
import MultiLineComposer from "./components/MultiLineComposer";
import ShellHistoryDrawer from "./components/ShellHistoryDrawer";
import ShellKnowledgeDrawer from "./components/ShellKnowledgeDrawer";
import ShellNuxDrawer from "./components/ShellNuxDrawer";
import { Tabs, TabList, TabPanels, Tab, TabPanel, useToast, Tooltip } from "@chakra-ui/react";
import { useState, useEffect, useCallback, useRef } from "react";
import { useMutation } from "@apollo/client";
import SshTerminal from "./components/SshTerminal";
import PtyTerminal from "./components/PtyTerminal";
import { CLOSE_PORTAL_MUTATION, CREATE_SHELL_MUTATION } from "./graphql";
import { SquareTerminal, Terminal, WifiOff, X } from "lucide-react";

interface PortalTab {
  id: string;
  type: string;
  target: string;
  pivotId?: string;
}

const NUX_STORAGE_KEY = "tavern_shell_nux_dismissed_v1";

const Shell = () => {
  const { shellId } = useParams<{ shellId: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  const {
    loading,
    error,
    shellData,
    beaconData,
    portalId,
    setPortalId,
    activeUsers,
    shellTasks,
    queuedTasks,
    runningTasks,
    stuckTask,
    stuckElapsedSeconds,
    pivots,
    hostCredentials,
    allCredentials,
    networkHosts,
    hostFiles,
    hostProcessCount,
    hostShells,
    lastKnowledgeSyncAt,
    refetchHistory,
    refetchKnowledgeGraph,
  } = useShellData(shellId);

  const { timeUntilCallback, isMissedCallback, isLateCheckin } = useCallbackTimer(beaconData);

  const [portalTabs, setPortalTabs] = useState<PortalTab[]>([]);
  const [tabIndex, setTabIndex] = useState(0);
  const [tabStatuses, setTabStatuses] = useState<
    Record<string, "connecting" | "connected" | "disconnected">
  >({});

  // Workspace drawers & composer state
  const [historyOpen, setHistoryOpen] = useState(false);
  const [knowledgeOpen, setKnowledgeOpen] = useState(false);
  const [nuxOpen, setNuxOpen] = useState(false);
  const [composerOpen, setComposerOpen] = useState(false);
  const [composerCode, setComposerCode] = useState<string>("");
  const [sshModalOpenExternal, setSshModalOpenExternal] = useState(false);
  const [sshModalInitialTarget, setSshModalInitialTarget] = useState("");
  const [showNuxBanner, setShowNuxBanner] = useState<boolean>(() => {
    try {
      return localStorage.getItem(NUX_STORAGE_KEY) !== "true";
    } catch {
      return true;
    }
  });

  const initializedPivotsShellRef = useRef<string | undefined>(undefined);
  const knownPivotIdsRef = useRef<Set<string>>(new Set());

  const handleDismissNuxBanner = useCallback(() => {
    setShowNuxBanner(false);
    try {
      localStorage.setItem(NUX_STORAGE_KEY, "true");
    } catch {
      // Ignore storage error
    }
  }, []);

  const handleTabConnectionStatusChange = useCallback(
    (tabId: string, status: "connecting" | "connected" | "disconnected") => {
      setTabStatuses((prev) => ({ ...prev, [tabId]: status }));
    },
    []
  );

  // Reset tab state when navigating between different shells
  useEffect(() => {
    setPortalTabs([]);
    setTabIndex(0);
    setTabStatuses({});
    initializedPivotsShellRef.current = undefined;
    knownPivotIdsRef.current = new Set();
  }, [shellId]);

  const handleOpenPortalTab = useCallback(
    (type: string, target: string, pivotId?: string, focusNewTab = true) => {
      const normalizedType = type.toLowerCase();
      const id = pivotId ? `pivot-${pivotId}` : `${normalizedType}-${target}-${Date.now()}`;
      setPortalTabs((prev) => {
        const existingIdx = prev.findIndex((t) => t.id === id);
        if (existingIdx !== -1) {
          if (focusNewTab) {
            setTabIndex(existingIdx + 1);
          }
          return prev;
        }
        const newTabs = [...prev, { id, type: normalizedType, target, pivotId }];
        if (focusNewTab) {
          setTabIndex(newTabs.length);
        }
        return newTabs;
      });
    },
    []
  );

  const handleClosePortalTab = useCallback((tabId: string, e?: React.MouseEvent) => {
    if (e) {
      e.stopPropagation();
    }
    setPortalTabs((prev) => {
      const idx = prev.findIndex((t) => t.id === tabId);
      if (idx === -1) return prev;
      const next = prev.filter((t) => t.id !== tabId);
      setTabIndex((currentTabIdx) => {
        if (currentTabIdx === idx + 1) {
          return Math.max(0, currentTabIdx - 1);
        }
        if (currentTabIdx > idx + 1) {
          return currentTabIdx - 1;
        }
        return currentTabIdx;
      });
      return next;
    });
  }, []);

  // Populate active pivots on initial load without hijacking focus from the main REPL tab,
  // and focus newly created pivots that appear after initial load.
  useEffect(() => {
    if (!shellData?.node?.pivots?.edges) return;
    const isInitialLoad = initializedPivotsShellRef.current !== shellId;
    initializedPivotsShellRef.current = shellId;

    shellData.node.pivots.edges.forEach((edge: any) => {
      const pivot = edge.node;
      if (!pivot?.id) return;
      const alreadyKnown = knownPivotIdsRef.current.has(pivot.id);
      knownPivotIdsRef.current.add(pivot.id);

      if (!pivot.closedAt && !alreadyKnown) {
        const label =
          pivot.kind?.toUpperCase() === "PTY" ? `PTY #${pivot.id}` : pivot.destination;
        handleOpenPortalTab(pivot.kind.toLowerCase(), label, pivot.id, !isInitialLoad);
      }
    });
  }, [shellData, shellId, handleOpenPortalTab]);

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
    localHistory,
    setShellInput,
    executeCode: submitShellCode,
    focusTerminal,
    sendCtrlC,
    sendCtrlR,
  } = useShellTerminal(
    shellId,
    loading,
    error,
    shellData,
    setPortalId,
    isLateCheckin,
    (type, target, pivotId) => handleOpenPortalTab(type, target, pivotId, true),
    () => {
      refetchHistory();
    }
  );

  const totalTabs = 1 + portalTabs.length;
  useTabHotkeys(totalTabs, setTabIndex);

  useEffect(() => {
    if (tabIndex === 0 && !composerOpen) {
      setTimeout(() => focusTerminal(), 0);
    }
  }, [tabIndex, composerOpen, focusTerminal]);

  const [closePortalMutation] = useMutation(CLOSE_PORTAL_MUTATION);
  const [createShellMutation, { loading: isSpawningShell }] = useMutation(CREATE_SHELL_MUTATION);

  const handleSpawnFreshShell = useCallback(async () => {
    const beaconId = beaconData?.id;
    if (!beaconId) return;
    try {
      const response = await createShellMutation({
        variables: { beaconId, data: "" },
      });
      const newShellId = response.data?.createShell?.id;
      if (newShellId) {
        toast({
          title: "Fresh Shell Created",
          description: `Switched to Shell #${newShellId} with a clean Eldritch interpreter.`,
          status: "success",
          duration: 4000,
          isClosable: true,
        });
        navigate(`/shells/${newShellId}`);
      }
    } catch (err: any) {
      toast({
        title: "Failed to create fresh shell",
        description: err?.message || "An error occurred while creating a new shell.",
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  }, [beaconData?.id, createShellMutation, navigate, toast]);

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
    setTabIndex(0);
    submitShellCode("portal.create()");
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
    handleOpenPortalTab("ssh", target, undefined, true);
  };

  const handlePtyOpen = () => {
    handleOpenPortalTab("pty", "PTY", undefined, true);
  };

  const handleLoadIntoComposer = useCallback((code: string) => {
    setTabIndex(0);
    setComposerCode(code);
    setComposerOpen(true);
  }, []);

  const handleRunCodeFromDrawer = useCallback(
    (code: string) => {
      setTabIndex(0);
      submitShellCode(code);
    },
    [submitShellCode]
  );

  const handleStageToPrompt = useCallback(
    (code: string) => {
      setTabIndex(0);
      setShellInput(code);
      setTimeout(() => focusTerminal(), 0);
    },
    [setShellInput, focusTerminal]
  );

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

  return (
    <AccessGate>
      <div className="flex flex-col h-screen p-3 md:p-4 bg-[#14161c] text-[#d4d4d4]">
        <ShellHeader
          shellData={shellData}
          activeUsers={activeUsers}
          portalId={portalId}
          onExport={handleExport}
          onNewPortal={handleNewPortal}
          onClosePortal={handleClosePortal}
          onSshConnect={handleSshConnect}
          onPtyOpen={handlePtyOpen}
          onSendCtrlC={sendCtrlC}
          onSendCtrlR={sendCtrlR}
          historyOpen={historyOpen}
          onToggleHistory={() => {
            setHistoryOpen((prev) => !prev);
            setKnowledgeOpen(false);
            setNuxOpen(false);
          }}
          historyCount={shellTasks.length}
          activeQueueCount={queuedTasks.length + runningTasks.length}
          knowledgeOpen={knowledgeOpen}
          onToggleKnowledge={() => {
            setKnowledgeOpen((prev) => !prev);
            setHistoryOpen(false);
            setNuxOpen(false);
          }}
          credentialCount={hostCredentials.length || allCredentials.length}
          credentials={allCredentials}
          hosts={networkHosts}
          sshModalOpenExternal={sshModalOpenExternal}
          sshModalInitialTarget={sshModalInitialTarget}
          onCloseSshModalExternal={() => {
            setSshModalOpenExternal(false);
            setSshModalInitialTarget("");
          }}
          onLoadIntoComposer={handleLoadIntoComposer}
          composerOpen={composerOpen}
          onToggleComposer={() => setComposerOpen((prev) => !prev)}
          nuxOpen={nuxOpen}
          onToggleNux={() => {
            setNuxOpen((prev) => !prev);
            setHistoryOpen(false);
            setKnowledgeOpen(false);
          }}
          onSpawnFreshShell={handleSpawnFreshShell}
        />

        <InterpreterStatusBanner
          closedAt={shellData?.node?.closedAt}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          stuckTask={
            stuckTask ? { task: stuckTask, elapsedSeconds: stuckElapsedSeconds } : null
          }
          queuedTasks={queuedTasks}
          runningTasks={runningTasks}
          timeUntilCallback={timeUntilCallback}
          onSpawnFreshShell={handleSpawnFreshShell}
          isSpawningShell={isSpawningShell}
          onOpenHistoryDrawer={() => {
            setHistoryOpen(true);
            setKnowledgeOpen(false);
            setNuxOpen(false);
          }}
          onOpenNuxDrawer={() => {
            setNuxOpen(true);
            setHistoryOpen(false);
            setKnowledgeOpen(false);
          }}
          showNuxWelcome={showNuxBanner}
          onDismissNuxWelcome={handleDismissNuxBanner}
          portalId={portalId}
          onOpenPortal={handleNewPortal}
        />

        {/* Main Workspace Row: Terminal/Pivots + Optional History or Guide Drawer */}
        <div className="flex-1 flex gap-3 min-h-0 overflow-hidden">
          <div className="flex-1 flex flex-col min-w-0 min-h-0 overflow-hidden">
            <Tabs
              index={tabIndex}
              onChange={(index) => setTabIndex(index)}
              variant="enclosed"
              flex="1"
              display="flex"
              flexDirection="column"
              overflow="hidden"
            >
              <TabList
                borderBottomColor="#2b303c"
                display={useTabs ? "flex" : "none"}
                overflowX="auto"
                overflowY="hidden"
                className="gap-1 pb-0.5"
              >
                <Tab
                  onMouseDown={(e) => e.preventDefault()}
                  _selected={{
                    color: "white",
                    bg: "#1e222d",
                    borderColor: "#343b4d",
                    borderBottomColor: "transparent",
                  }}
                  color="#9ca3af"
                  borderColor="transparent"
                  className="text-xs font-medium px-3 py-1.5 rounded-t-md"
                >
                  <span className="flex items-center gap-1.5">
                    <span
                      className={`w-2 h-2 rounded-full ${
                        connectionStatus === "connected"
                          ? "bg-emerald-400"
                          : connectionStatus === "reconnecting"
                          ? "bg-amber-400 animate-pulse"
                          : "bg-red-500"
                      }`}
                    />
                    <span>{shellData?.node?.beacon?.name ?? "Eldritch Shell"}</span>
                  </span>
                </Tab>

                {portalTabs.map((tab) => {
                  const status = tabStatuses[tab.id];
                  const isDisconnected = status === "disconnected";
                  const isConnecting = status === "connecting";
                  const isSsh = tab.type.toLowerCase() === "ssh";

                  return (
                    <Tab
                      key={tab.id}
                      onMouseDown={(e) => e.preventDefault()}
                      _selected={{
                        color: "white",
                        bg: "#1e222d",
                        borderColor: "#343b4d",
                        borderBottomColor: "transparent",
                      }}
                      color="#9ca3af"
                      borderColor="transparent"
                      className="text-xs font-medium px-3 py-1.5 rounded-t-md group"
                    >
                      <span className="flex items-center gap-1.5">
                        {isDisconnected ? (
                          <Tooltip label="Terminal session ended" hasArrow>
                            <span className="text-red-400 flex items-center gap-1">
                              <WifiOff size={13} />
                            </span>
                          </Tooltip>
                        ) : isSsh ? (
                          <Terminal
                            size={13}
                            className={
                              isConnecting ? "text-amber-400 animate-pulse" : "text-purple-400"
                            }
                          />
                        ) : (
                          <SquareTerminal
                            size={13}
                            className={
                              isConnecting ? "text-amber-400 animate-pulse" : "text-emerald-400"
                            }
                          />
                        )}

                        <span className={isDisconnected ? "line-through text-gray-400" : ""}>
                          {tab.target}
                        </span>

                        {isDisconnected && (
                          <span className="px-1.5 py-0.2 rounded bg-red-500/20 text-red-300 text-[10px] font-semibold">
                            Ended
                          </span>
                        )}

                        <span
                          role="button"
                          tabIndex={0}
                          aria-label={`Close ${tab.target} tab`}
                          onClick={(e) => handleClosePortalTab(tab.id, e)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.preventDefault();
                              handleClosePortalTab(tab.id);
                            }
                          }}
                          className="ml-1 p-0.5 rounded hover:bg-[#343b4d] text-gray-400 hover:text-white transition-colors"
                        >
                          <X size={12} />
                        </span>
                      </span>
                    </Tab>
                  );
                })}
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
                  <MultiLineComposer
                    isOpen={composerOpen}
                    onToggle={() => setComposerOpen((prev) => !prev)}
                    code={composerCode}
                    onChange={setComposerCode}
                    onRunCode={(code) => submitShellCode(code)}
                    onStageToPrompt={(code) => {
                      setShellInput(code);
                      setComposerOpen(false);
                      setTimeout(() => focusTerminal(), 0);
                    }}
                    disabled={!!shellData?.node?.closedAt}
                  />
                </TabPanel>

                {portalTabs.map((tab, idx) => (
                  <TabPanel
                    key={tab.id}
                    flex="1"
                    p={0}
                    display="flex"
                    flexDirection="column"
                    overflow="hidden"
                  >
                    {tab.type === "ssh" && (portalId || tab.pivotId) && (
                      <SshTerminal
                        portalId={portalId || 0}
                        target={tab.target}
                        pivotId={tab.pivotId ? parseInt(tab.pivotId, 10) : undefined}
                        shellId={shellId || ""}
                        isActive={tabIndex === idx + 1}
                        onConnectionStatusChange={(status) =>
                          handleTabConnectionStatusChange(tab.id, status)
                        }
                        onCloseTab={() => handleClosePortalTab(tab.id)}
                      />
                    )}
                    {tab.type === "pty" && (portalId || tab.pivotId) && (
                      <PtyTerminal
                        portalId={portalId || 0}
                        pivotId={tab.pivotId ? parseInt(tab.pivotId, 10) : undefined}
                        shellId={shellId || ""}
                        isActive={tabIndex === idx + 1}
                        onConnectionStatusChange={(status) =>
                          handleTabConnectionStatusChange(tab.id, status)
                        }
                        onCloseTab={() => handleClosePortalTab(tab.id)}
                      />
                    )}
                  </TabPanel>
                ))}
              </TabPanels>
            </Tabs>
          </div>

          <ShellKnowledgeDrawer
            isOpen={knowledgeOpen}
            onClose={() => setKnowledgeOpen(false)}
            currentShellId={shellId}
            currentHostId={shellData?.node?.beacon?.host?.id}
            currentHostName={shellData?.node?.beacon?.host?.name}
            currentHostIp={shellData?.node?.beacon?.host?.primaryIP}
            portalId={portalId}
            hostCredentials={hostCredentials}
            allCredentials={allCredentials}
            networkHosts={networkHosts}
            hostFiles={hostFiles}
            hostProcessCount={hostProcessCount}
            hostShells={hostShells}
            lastKnowledgeSyncAt={lastKnowledgeSyncAt}
            onRefetchKnowledgeGraph={() => refetchKnowledgeGraph()}
            onOpenSshWithTarget={(target) => {
              setSshModalInitialTarget(target);
              setSshModalOpenExternal(true);
            }}
            onLoadIntoComposer={handleLoadIntoComposer}
            onStageToPrompt={handleStageToPrompt}
          />

          <ShellHistoryDrawer
            isOpen={historyOpen}
            onClose={() => setHistoryOpen(false)}
            shellTasks={shellTasks}
            localHistory={localHistory}
            pivots={pivots}
            onRunCode={handleRunCodeFromDrawer}
            onLoadIntoComposer={handleLoadIntoComposer}
            onOpenPivotTab={(type, target, pivotId) =>
              handleOpenPortalTab(type, target, pivotId, true)
            }
          />

          <ShellNuxDrawer
            isOpen={nuxOpen}
            onClose={() => setNuxOpen(false)}
            portalId={portalId}
            onRunCode={handleRunCodeFromDrawer}
            onStageToPrompt={handleStageToPrompt}
            onLoadIntoComposer={handleLoadIntoComposer}
            onSpawnFreshShell={handleSpawnFreshShell}
          />
        </div>

        <ShellStatusBar
          portalId={portalId}
          timeUntilCallback={timeUntilCallback}
          isMissedCallback={isMissedCallback}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          closedAt={shellData?.node?.closedAt}
          queuedCount={queuedTasks.length}
          runningCount={runningTasks.length}
          onOpenHistory={() => {
            setHistoryOpen(true);
            setNuxOpen(false);
          }}
        />
      </div>
    </AccessGate>
  );
};

export default Shell;
