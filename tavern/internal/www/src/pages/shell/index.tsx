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
import MultiLineComposer, {
  hasShellExecutedRc,
  loadRcEnabled,
  loadRcScript,
  markShellExecutedRc,
} from "./components/MultiLineComposer";
import SessionSettings from "./components/SessionSettings";
import ShellHistoryDrawer from "./components/ShellHistoryDrawer";
import ShellPivotsDrawer from "./components/ShellPivotsDrawer";
import ShellKnowledgeDrawer from "./components/ShellKnowledgeDrawer";
import ShellNuxDrawer from "./components/ShellNuxDrawer";
import ShellWalkthrough from "./components/ShellWalkthrough";
import { Tabs, TabList, TabPanels, Tab, TabPanel, useToast, Tooltip } from "@chakra-ui/react";
import { useState, useEffect, useCallback, useRef } from "react";
import { useMutation } from "@apollo/client";
import SshTerminal from "./components/SshTerminal";
import PtyTerminal from "./components/PtyTerminal";
import { CLOSE_PORTAL_MUTATION, CREATE_SHELL_MUTATION } from "./graphql";
import { SquareTerminal, Terminal, WifiOff, X } from "lucide-react";

export interface PortalTab {
  id: string;
  type: string;
  target: string;
  pivotId?: string;
}

export function formatPivotTabLabel(tab: Pick<PortalTab, "type" | "target">): string {
  if (tab.type.toLowerCase() === "ssh") {
    const raw = (tab.target || "").trim();
    if (!raw) return "SSH";
    const afterAt = raw.includes("@") ? raw.slice(raw.lastIndexOf("@") + 1) : raw;
    const hostOnly = afterAt.replace(/:\d+$/, "");
    return hostOnly || afterAt || raw;
  }
  const cleaned = (tab.target || "PTY").replace(/^PTY\s*#\d+$/i, "PTY").trim();
  if (!cleaned || cleaned.toLowerCase() === "exit") return "PTY";
  return cleaned.length > 24 ? `${cleaned.slice(0, 24)}...` : cleaned;
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
    allShellTasks,
    queuedTasks,
    runningTasks,
    stuckTask,
    stuckElapsedSeconds,
    pivots,
    tomes,
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
  const [pivotsOpen, setPivotsOpen] = useState(false);
  const [knowledgeOpen, setKnowledgeOpen] = useState(false);
  const [nuxOpen, setNuxOpen] = useState(false);
  const [composerOpen, setComposerOpen] = useState(false);
  const [sessionSettingsOpen, setSessionSettingsOpen] = useState(false);
  const [composerCode, setComposerCode] = useState<string>("");
  const [rcCode, setRcCode] = useState<string>(() => loadRcScript());
  const [rcEnabled, setRcEnabled] = useState<boolean>(() => loadRcEnabled());
  const [actionsMenuTourOpen, setActionsMenuTourOpen] = useState(false);
  const [queueTourOpen, setQueueTourOpen] = useState(false);
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

  const handleBindTabPivotId = useCallback((tabId: string, createdPivotId: string) => {
    knownPivotIdsRef.current.add(createdPivotId);
    setPortalTabs((prev) =>
      prev.map((t) => {
        if (t.id !== tabId) return t;
        return {
          ...t,
          pivotId: createdPivotId,
        };
      })
    );
  }, []);

  const handleUpdateTabTarget = useCallback((tabId: string, nextTarget: string) => {
    if (nextTarget.trim().toLowerCase() === "exit") return;
    setPortalTabs((prev) =>
      prev.map((t) => (t.id === tabId ? { ...t, target: nextTarget } : t))
    );
  }, []);

  const handleOpenPortalTab = useCallback(
    (type: string, target: string, pivotId?: string, focusNewTab = true) => {
      const normalizedType = type.toLowerCase();
      const cleanedTarget =
        normalizedType === "pty"
          ? (target || "PTY").replace(/^PTY\s*#\d+$/i, "PTY")
          : target;
      if (pivotId) {
        knownPivotIdsRef.current.add(String(pivotId));
      }
      setPortalTabs((prev) => {
        // 1. Check if a tab already exists for this exact pivotId or deterministic id
        const deterministicId = pivotId ? `pivot-${pivotId}` : undefined;
        const existingByPivotIdx = prev.findIndex(
          (t) =>
            (pivotId && String(t.pivotId) === String(pivotId)) ||
            (deterministicId && t.id === deterministicId)
        );
        if (existingByPivotIdx !== -1) {
          if (focusNewTab) {
            setTabIndex(existingByPivotIdx + 1);
          }
          return prev;
        }

        // 2. If a pivotId arrived (e.g. from GraphQL poll) for an already-open tab that hasn't bound its pivotId yet,
        // bind the pivotId in place without changing tab.id (preserving the mounted terminal & WebSocket).
        if (pivotId) {
          const pendingIdx = prev.findIndex(
            (t) =>
              !t.pivotId &&
              t.type === normalizedType &&
              (normalizedType === "pty" || t.target === target)
          );
          if (pendingIdx !== -1) {
            const next = [...prev];
            next[pendingIdx] = {
              ...next[pendingIdx],
              pivotId: String(pivotId),
              target: next[pendingIdx].target || cleanedTarget,
            };
            if (focusNewTab) {
              setTabIndex(pendingIdx + 1);
            }
            return next;
          }
        }

        const id = deterministicId || `${normalizedType}-${cleanedTarget}-${Date.now()}`;
        const newTabs = [
          ...prev,
          {
            id,
            type: normalizedType,
            target: cleanedTarget,
            pivotId: pivotId ? String(pivotId) : undefined,
          },
        ];
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
          pivot.kind?.toUpperCase() === "PTY" ? "PTY" : pivot.destination;
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
    isReplReady,
    isInitialConnecting,
    handleTooltipMouseEnter,
    handleTooltipMouseLeave,
    getSessionInputs,
    executeCode: submitShellCode,
    focusTerminal,
    sendCtrlC,
    sendCtrlR,
    reconnect,
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
    },
    allShellTasks.length > 0 ? allShellTasks : shellTasks
  );

  // Automatically execute the shared RC script the first time a user connects to a shell when RC is toggled on
  useEffect(() => {
    if (
      !shellId ||
      !rcEnabled ||
      !rcCode.trim() ||
      !isReplReady ||
      connectionStatus !== "connected" ||
      shellData?.node?.closedAt ||
      hasShellExecutedRc(shellId)
    ) {
      return;
    }
    if (submitShellCode(rcCode)) {
      markShellExecutedRc(shellId);
    }
  }, [
    shellId,
    rcEnabled,
    rcCode,
    isReplReady,
    connectionStatus,
    shellData?.node?.closedAt,
    submitShellCode,
  ]);

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
    const beaconId =
      shellData?.node?.beacon?.id || beaconData?.node?.id || beaconData?.id;
    if (!beaconId) {
      toast({
        title: "Cannot spawn new shell",
        description: "Beacon ID is not available for this shell session.",
        status: "warning",
        duration: 4000,
        isClosable: true,
      });
      return;
    }
    try {
      const response = await createShellMutation({
        variables: { beaconId },
      });
      const newShellId = response.data?.createShell?.id;
      if (newShellId) {
        toast({
          title: "New shell created",
          description: `Switched to Shell #${newShellId} with a clean Eldritch interpreter.`,
          status: "success",
          duration: 4000,
          isClosable: true,
        });
        navigate(`/shells/${newShellId}`);
      }
    } catch (err: any) {
      toast({
        title: "Failed to spawn new shell",
        description: err?.message || "An error occurred while creating a new shell.",
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  }, [
    shellData?.node?.beacon?.id,
    beaconData?.node?.id,
    beaconData?.id,
    createShellMutation,
    navigate,
    toast,
  ]);

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

  if (connectionError) {
    return (
      <div style={{ padding: "20px" }}>
        <AlertError label="Shell Connection Failed" details={connectionError} />
      </div>
    );
  }

  const isConnecting = (loading || isInitialConnecting) && !shellData?.node?.closedAt;
  const useTabs = portalTabs.length > 0;

  return (
    <AccessGate>
      <div className="shell-font-onest flex flex-col h-screen w-full overflow-hidden bg-[#11131a] text-[#d4d4d4]">
        <link
          rel="stylesheet"
          href="https://fonts.googleapis.com/css2?family=Onest:wght@100..900&display=swap"
        />
        <ShellWalkthrough
          isOpen={showNuxBanner && !isConnecting && !shellData?.node?.closedAt}
          onDismiss={handleDismissNuxBanner}
          onOpenComposer={() => {
            setComposerOpen(true);
            setSessionSettingsOpen(false);
          }}
          onCloseComposer={() => {
            setComposerOpen(false);
          }}
          onOpenSessionSettings={() => {
            setSessionSettingsOpen(true);
            setComposerOpen(false);
          }}
          onCloseSessionSettings={() => {
            setSessionSettingsOpen(false);
          }}
          onPortalStepChange={setActionsMenuTourOpen}
          onQueueStepChange={setQueueTourOpen}
          onOpenGuideDrawer={() => {
            setNuxOpen(true);
            setHistoryOpen(false);
            setPivotsOpen(false);
            setKnowledgeOpen(false);
          }}
        />
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
            setPivotsOpen(false);
            setKnowledgeOpen(false);
            setNuxOpen(false);
          }}
          historyCount={shellTasks.length}
          activeQueueCount={queuedTasks.length + runningTasks.length}
          pivotsOpen={pivotsOpen}
          onTogglePivots={() => {
            setPivotsOpen((prev) => !prev);
            setHistoryOpen(false);
            setKnowledgeOpen(false);
            setNuxOpen(false);
          }}
          pivotCount={pivots.filter((p) => !p.closedAt).length}
          knowledgeOpen={knowledgeOpen}
          onToggleKnowledge={() => {
            setKnowledgeOpen((prev) => !prev);
            setHistoryOpen(false);
            setPivotsOpen(false);
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
          onToggleComposer={() => {
            setComposerOpen((prev) => {
              const next = !prev;
              if (next) setSessionSettingsOpen(false);
              return next;
            });
          }}
          nuxOpen={nuxOpen}
          onToggleNux={() => {
            setNuxOpen((prev) => !prev);
            setHistoryOpen(false);
            setPivotsOpen(false);
            setKnowledgeOpen(false);
          }}
          onSpawnFreshShell={handleSpawnFreshShell}
          actionsMenuTourOpen={actionsMenuTourOpen}
        />

        <InterpreterStatusBanner
          closedAt={shellData?.node?.closedAt}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          isConnecting={isConnecting}
          stuckTask={
            stuckTask ? { task: stuckTask, elapsedSeconds: stuckElapsedSeconds } : null
          }
          queuedTasks={queuedTasks}
          runningTasks={runningTasks}
          timeUntilCallback={timeUntilCallback}
          onSpawnFreshShell={handleSpawnFreshShell}
          isSpawningShell={isSpawningShell}
          onReconnect={reconnect}
        />

        {/* Main Workspace Row: Terminal/Pivots + Optional History or Guide Drawer */}
        <div className="flex-1 flex min-h-0 overflow-hidden">
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
                          : isConnecting || connectionStatus === "reconnecting"
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
                          {formatPivotTabLabel(tab)}
                        </span>

                        {isDisconnected && (
                          <span className="px-1.5 py-0.2 rounded bg-red-500/20 text-red-300 text-[10px] font-semibold">
                            Ended
                          </span>
                        )}

                        <span
                          role="button"
                          tabIndex={0}
                          aria-label={`Close ${formatPivotTabLabel(tab)} tab`}
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
                    isConnecting={isConnecting}
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
                    disabled={!!shellData?.node?.closedAt}
                    tomes={tomes}
                  />
                  <SessionSettings
                    isOpen={sessionSettingsOpen}
                    onToggle={() => setSessionSettingsOpen((prev) => !prev)}
                    disabled={!!shellData?.node?.closedAt}
                    tomes={tomes}
                    rcCode={rcCode}
                    onRcCodeChange={setRcCode}
                    rcEnabled={rcEnabled}
                    onRcEnabledChange={setRcEnabled}
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
                        onPivotCreated={(createdPivotId) =>
                          handleBindTabPivotId(tab.id, String(createdPivotId))
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
                        onPivotCreated={(createdPivotId) =>
                          handleBindTabPivotId(tab.id, String(createdPivotId))
                        }
                        onLastCommandChange={(cmd) => handleUpdateTabTarget(tab.id, cmd)}
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
          />

          <ShellPivotsDrawer
            isOpen={pivotsOpen}
            onClose={() => setPivotsOpen(false)}
            portalId={portalId}
            pivots={pivots}
            onNewPortal={handleNewPortal}
            onClosePortal={handleClosePortal}
            onOpenPty={handlePtyOpen}
            onOpenSshModal={() => {
              setSshModalInitialTarget("");
              setSshModalOpenExternal(true);
            }}
            onOpenPivotTab={(type, target, pivotId) =>
              handleOpenPortalTab(type, target, pivotId, true)
            }
          />

          <ShellHistoryDrawer
            isOpen={historyOpen}
            onClose={() => setHistoryOpen(false)}
            shellTasks={shellTasks}
            allShellTasks={allShellTasks}
            currentShellId={shellId}
            onRunCode={handleRunCodeFromDrawer}
            onLoadIntoComposer={handleLoadIntoComposer}
          />

          <ShellNuxDrawer
            isOpen={nuxOpen}
            onClose={() => setNuxOpen(false)}
            portalId={portalId}
            onRunCode={handleRunCodeFromDrawer}
            onLoadIntoComposer={handleLoadIntoComposer}
            onSpawnFreshShell={handleSpawnFreshShell}
            onStartWalkthrough={() => {
              setNuxOpen(false);
              setHistoryOpen(false);
              setPivotsOpen(false);
              setKnowledgeOpen(false);
              setShowNuxBanner(true);
            }}
          />
        </div>

        <ShellStatusBar
          portalId={portalId}
          timeUntilCallback={timeUntilCallback}
          isMissedCallback={isMissedCallback}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          isConnecting={isConnecting}
          closedAt={shellData?.node?.closedAt}
          queuedCount={queuedTasks.length}
          runningCount={runningTasks.length}
          queuedTasks={queuedTasks}
          runningTasks={runningTasks}
          queueTourOpen={queueTourOpen}
          sessionSettingsOpen={sessionSettingsOpen}
          onToggleSessionSettings={() => {
            setSessionSettingsOpen((prev) => {
              const next = !prev;
              if (next) setComposerOpen(false);
              return next;
            });
          }}
          onReconnect={reconnect}
        />
      </div>
    </AccessGate>
  );
};

export default Shell;
