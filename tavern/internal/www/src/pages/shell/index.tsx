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
  loadShellFontSize,
  markShellExecutedRc,
} from "./components/MultiLineComposer";
import SessionSettings from "./components/SessionSettings";
import ShellScratchpad from "./components/ShellScratchpad";
import ShellHistoryDrawer from "./components/ShellHistoryDrawer";
import ShellKnowledgeDrawer from "./components/ShellKnowledgeDrawer";
import ShellSidebar from "./components/ShellSidebar";
import ShellWalkthrough from "./components/ShellWalkthrough";
import { Tabs, TabList, TabPanels, Tab, TabPanel, useToast, Tooltip } from "@chakra-ui/react";
import { useState, useEffect, useCallback, useRef } from "react";
import { useMutation } from "@apollo/client";
import SshTerminal from "./components/SshTerminal";
import PtyTerminal from "./components/PtyTerminal";
import TabContextMenu from "./components/TabContextMenu";
import RenameTabModal from "./components/RenameTabModal";
import { CLOSE_PORTAL_MUTATION, CREATE_SHELL_MUTATION } from "./graphql";
import { formatPivotTabLabel } from "./utils/networkGraphUtils";
import { DEFAULT_RC_ALIASES_SCRIPT, registerUserSymbolsFromCode } from "./utils/eldritchLsp";
import { Minus, SquareTerminal, Terminal, WifiOff } from "lucide-react";

export { formatPivotTabLabel };

export interface PortalTab {
  id: string;
  type: string;
  target: string;
  pivotId?: string;
  credentialId?: string;
  customTitle?: string;
}

const NUX_STORAGE_KEY = "tavern_shell_nux_dismissed_v1";
const PIVOT_TARGETS_STORAGE_KEY = "tavern_shell_pivot_targets_v1";
const PIVOT_CUSTOM_TITLES_STORAGE_KEY = "tavern_shell_pivot_custom_titles_v1";

const getMainTabIndicatorClass = (
  connectionStatus: string,
  isConnecting: boolean
): string => {
  if (connectionStatus === "connected") {
    return "bg-emerald-400";
  }
  if (isConnecting || connectionStatus === "reconnecting") {
    return "bg-amber-400 animate-pulse";
  }
  return "bg-red-500";
};

const PortalTabIcon: React.FC<{
  isDisconnected: boolean;
  isConnecting: boolean;
  isSsh: boolean;
}> = ({ isDisconnected, isConnecting, isSsh }) => {
  if (isDisconnected) {
    return (
      <Tooltip label="Terminal session ended" hasArrow>
        <span className="text-red-400 flex items-center gap-1">
          <WifiOff className="w-3.5 h-3.5" />
        </span>
      </Tooltip>
    );
  }
  if (isSsh) {
    return (
      <Terminal
        className={`w-3.5 h-3.5 ${isConnecting ? "text-amber-400 animate-pulse" : "text-purple-400"}`}
      />
    );
  }
  return (
    <SquareTerminal
      className={`w-3.5 h-3.5 ${isConnecting ? "text-amber-400 animate-pulse" : "text-emerald-400"}`}
    />
  );
};

const ShellContent = () => {
  const { shellId } = useParams<{ shellId: string }>();
  const navigate = useNavigate();
  const toast = useToast();

  // Workspace drawers & composer state
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [knowledgeOpen, setKnowledgeOpen] = useState(false);

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
    refetchTomes,
    hostCredentials,
    allCredentials,
    networkHosts,
    hostFiles,
    hostProcessCount,
    hostShells,
    lastKnowledgeSyncAt,
    refetchHistory,
    refetchKnowledgeGraph,
    refetchShell,
  } = useShellData(shellId, { historyOpen, knowledgeOpen });

  const { timeUntilCallback, isMissedCallback, isLateCheckin, callbackStats } = useCallbackTimer(
    beaconData,
    shellData?.node?.beacon?.interval
  );

  const [portalTabs, setPortalTabs] = useState<PortalTab[]>([]);
  const [tabIndex, setTabIndex] = useState(0);
  const [tabStatuses, setTabStatuses] = useState<
    Record<string, "connecting" | "connected" | "disconnected">
  >({});

  const [activeBottomPanel, setActiveBottomPanel] = useState<
    "composer" | "settings" | "scratchpad" | null
  >(null);
  const composerOpen = activeBottomPanel === "composer";
  const sessionSettingsOpen = activeBottomPanel === "settings";
  const scratchpadOpen = activeBottomPanel === "scratchpad";
  const [composerCode, setComposerCode] = useState<string>("");
  const [rcCode, setRcCode] = useState<string>(() => loadRcScript());
  const [rcEnabled, setRcEnabled] = useState<boolean>(() => loadRcEnabled());
  const [shellFontSize, setShellFontSize] = useState<number>(() => loadShellFontSize());
  const [actionsMenuTourOpen, setActionsMenuTourOpen] = useState(false);
  const [queueTourOpen, setQueueTourOpen] = useState(false);
  const [sshModalOpenExternal, setSshModalOpenExternal] = useState(false);
  const [sshModalInitialTarget, setSshModalInitialTarget] = useState("");
  const [sshModalInitialCredentialId, setSshModalInitialCredentialId] = useState<string | undefined>(
    undefined
  );
  const [showNuxBanner, setShowNuxBanner] = useState<boolean>(() => {
    try {
      return localStorage.getItem(NUX_STORAGE_KEY) !== "true";
    } catch {
      return true;
    }
  });

  const initializedPivotsShellRef = useRef<string | undefined>(undefined);
  const knownPivotIdsRef = useRef<Set<string>>(new Set());
  const [pivotTargets, setPivotTargets] = useState<Record<string, string>>(() => {
    try {
      const raw = sessionStorage.getItem(PIVOT_TARGETS_STORAGE_KEY);
      return raw ? JSON.parse(raw) : {};
    } catch {
      return {};
    }
  });
  const pivotTargetsRef = useRef<Record<string, string>>(pivotTargets);
  pivotTargetsRef.current = pivotTargets;

  const [customTitles, setCustomTitles] = useState<Record<string, string>>(() => {
    try {
      const raw = sessionStorage.getItem(PIVOT_CUSTOM_TITLES_STORAGE_KEY);
      return raw ? JSON.parse(raw) : {};
    } catch {
      return {};
    }
  });
  const customTitlesRef = useRef<Record<string, string>>(customTitles);
  customTitlesRef.current = customTitles;

  const recordCustomTitle = useCallback((key: string, title?: string) => {
    if (!key) return;
    setCustomTitles((prev) => {
      const next = { ...prev };
      if (title && title.trim()) {
        next[key] = title.trim();
      } else {
        delete next[key];
      }
      customTitlesRef.current = next;
      try {
        sessionStorage.setItem(PIVOT_CUSTOM_TITLES_STORAGE_KEY, JSON.stringify(next));
      } catch {
        // Ignore storage error
      }
      return next;
    });
  }, []);

  const [contextMenu, setContextMenu] = useState<{
    isOpen: boolean;
    x: number;
    y: number;
    tab: PortalTab | null;
  }>({
    isOpen: false,
    x: 0,
    y: 0,
    tab: null,
  });

  const [renameModalTab, setRenameModalTab] = useState<PortalTab | null>(null);

  const handleRenameTab = useCallback(
    (tabId: string, newTitle: string) => {
      const trimmed = newTitle.trim();
      setPortalTabs((prev) =>
        prev.map((t) => {
          if (t.id !== tabId) return t;
          if (t.pivotId) {
            recordCustomTitle(String(t.pivotId), trimmed || undefined);
          }
          recordCustomTitle(t.id, trimmed || undefined);
          return {
            ...t,
            customTitle: trimmed || undefined,
          };
        })
      );
    },
    [recordCustomTitle]
  );

  const handleResetTabName = useCallback(
    (tabId: string) => {
      setPortalTabs((prev) =>
        prev.map((t) => {
          if (t.id !== tabId) return t;
          if (t.pivotId) {
            recordCustomTitle(String(t.pivotId), undefined);
          }
          recordCustomTitle(t.id, undefined);
          return {
            ...t,
            customTitle: undefined,
          };
        })
      );
    },
    [recordCustomTitle]
  );
  const lastUnboundPtyTargetRef = useRef<string>("");

  const recordPivotTarget = useCallback((pivotIdStr: string, target: string) => {
    if (!pivotIdStr || !target || target.toUpperCase() === "PTY") return;
    setPivotTargets((prev) => {
      if (prev[pivotIdStr] === target) return prev;
      const next = { ...prev, [pivotIdStr]: target };
      pivotTargetsRef.current = next;
      try {
        sessionStorage.setItem(PIVOT_TARGETS_STORAGE_KEY, JSON.stringify(next));
      } catch {
        // Ignore storage error
      }
      return next;
    });
  }, []);

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
    lastUnboundPtyTargetRef.current = "";
  }, [shellId]);

  const handleBindTabPivotId = useCallback(
    (tabId: string, createdPivotId: string) => {
      knownPivotIdsRef.current.add(createdPivotId);
      setPortalTabs((prev) =>
        prev.map((t) => {
          if (t.id !== tabId) return t;
          const targetToSave =
            t.target && t.target !== "PTY" ? t.target : lastUnboundPtyTargetRef.current;
          if (t.customTitle) {
            recordCustomTitle(String(createdPivotId), t.customTitle);
          }
          if (targetToSave) {
            recordPivotTarget(String(createdPivotId), targetToSave);
            if (lastUnboundPtyTargetRef.current === targetToSave) {
              lastUnboundPtyTargetRef.current = "";
            }
          }
          return {
            ...t,
            target: targetToSave || t.target,
            pivotId: createdPivotId,
          };
        })
      );
      refetchHistory();
      refetchKnowledgeGraph();
    },
    [recordPivotTarget, recordCustomTitle, refetchHistory, refetchKnowledgeGraph]
  );

  const handleUpdateTabTarget = useCallback(
    (tabId: string, nextTarget: string) => {
      if (nextTarget.trim().toLowerCase() === "exit") return;
      setPortalTabs((prev) => {
        const matched = prev.find((t) => t.id === tabId);
        if (matched?.customTitle) {
          return prev;
        }
        if (matched?.pivotId) {
          recordPivotTarget(String(matched.pivotId), nextTarget);
        } else if (matched?.type === "pty") {
          lastUnboundPtyTargetRef.current = nextTarget;
        }
        return prev.map((t) => (t.id === tabId ? { ...t, target: nextTarget } : t));
      });
    },
    [recordPivotTarget]
  );

  const handleOpenPortalTab = useCallback(
    (
      type: string,
      target: string,
      pivotId?: string,
      focusNewTab = true,
      credentialId?: string
    ) => {
      const normalizedType = type.toLowerCase();
      const rememberedTarget =
        pivotId && normalizedType === "pty"
          ? pivotTargetsRef.current[String(pivotId)]
          : undefined;
      const rememberedCustomTitle =
        pivotId ? customTitlesRef.current[String(pivotId)] : undefined;
      const cleanedTarget =
        normalizedType === "pty"
          ? rememberedTarget || (target || "PTY").replace(/^PTY\s*#\d+$/i, "PTY")
          : target;
      if (pivotId) {
        knownPivotIdsRef.current.add(String(pivotId));
        if (cleanedTarget && cleanedTarget !== "PTY" && normalizedType === "pty") {
          recordPivotTarget(String(pivotId), cleanedTarget);
        }
        if (rememberedCustomTitle) {
          recordCustomTitle(String(pivotId), rememberedCustomTitle);
        }
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
            const resolvedTarget =
              next[pendingIdx].target && next[pendingIdx].target !== "PTY"
                ? next[pendingIdx].target
                : cleanedTarget;
            if (normalizedType === "pty" && resolvedTarget && resolvedTarget !== "PTY") {
              recordPivotTarget(String(pivotId), resolvedTarget);
            }
            const resolvedCustomTitle =
              next[pendingIdx].customTitle || rememberedCustomTitle;
            if (resolvedCustomTitle) {
              recordCustomTitle(String(pivotId), resolvedCustomTitle);
            }
            next[pendingIdx] = {
              ...next[pendingIdx],
              pivotId: String(pivotId),
              target: resolvedTarget,
              customTitle: resolvedCustomTitle,
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
            credentialId,
            customTitle: rememberedCustomTitle,
          },
        ];
        if (focusNewTab) {
          setTabIndex(newTabs.length);
        }
        return newTabs;
      });
    },
    [recordPivotTarget, recordCustomTitle]
  );

  const handleClosePortalTab = useCallback(
    (tabId: string, e?: React.MouseEvent) => {
      if (e) {
        e.stopPropagation();
      }
      setPortalTabs((prev) => {
        const idx = prev.findIndex((t) => t.id === tabId);
        if (idx === -1) return prev;
        const closingTab = prev[idx];
        if (closingTab?.pivotId && closingTab.target && closingTab.target !== "PTY") {
          recordPivotTarget(String(closingTab.pivotId), closingTab.target);
        } else if (
          closingTab?.type === "pty" &&
          closingTab.target &&
          closingTab.target !== "PTY"
        ) {
          lastUnboundPtyTargetRef.current = closingTab.target;
        }
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
    },
    [recordPivotTarget]
  );

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
    promptPos,
    isAwaitingCommand,
    completedTaskIds,
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
      refetchKnowledgeGraph();
    },
    allShellTasks.length > 0 ? allShellTasks : shellTasks,
    shellFontSize
  );

  const completedTaskIdSet = new Set(completedTaskIds);
  const activeQueuedTasks = queuedTasks.filter((t) => !completedTaskIdSet.has(String(t.id)));
  const activeRunningTasks = runningTasks.filter((t) => !completedTaskIdSet.has(String(t.id)));
  const isPromptBusy =
    activeQueuedTasks.length > 0 || activeRunningTasks.length > 0 || isAwaitingCommand;

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

  // Keep autocomplete aware of user defined functions and variables in RC script
  useEffect(() => {
    const codeToRegister = rcCode && rcCode.trim() ? rcCode : DEFAULT_RC_ALIASES_SCRIPT;
    registerUserSymbolsFromCode(codeToRegister);
  }, [rcCode]);

  const totalTabs = 1 + portalTabs.length;
  useTabHotkeys(totalTabs, setTabIndex);

  useEffect(() => {
    if (tabIndex === 0 && !composerOpen && !scratchpadOpen) {
      setTimeout(() => focusTerminal(), 0);
    }
  }, [tabIndex, composerOpen, scratchpadOpen, focusTerminal]);

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

  const handleSshConnect = (target: string, credentialId?: string) => {
    handleOpenPortalTab("ssh", target, undefined, true, credentialId);
  };

  const handlePtyOpen = () => {
    handleOpenPortalTab("pty", "PTY", undefined, true);
  };

  const handleLoadIntoComposer = useCallback((code: string) => {
    setTabIndex(0);
    setComposerCode(code);
    setActiveBottomPanel("composer");
  }, []);

  const handleRunCodeFromDrawer = useCallback(
    (code: string) => {
      setTabIndex(0);
      submitShellCode(code);
    },
    [submitShellCode]
  );

  const handleReconnect = useCallback(() => {
    if (!shellData?.node?.id) {
      refetchShell().catch(() => {});
    }
    reconnect();
  }, [shellData?.node?.id, refetchShell, reconnect]);

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
    <div className="shell-font-onest flex flex-col h-screen w-full overflow-hidden bg-[#11131a] text-[#d4d4d4]">
      <ShellWalkthrough
          isOpen={showNuxBanner && !isConnecting && !shellData?.node?.closedAt}
          onDismiss={handleDismissNuxBanner}
          onOpenComposer={() => {
            setActiveBottomPanel("composer");
          }}
          onCloseComposer={() => {
            setActiveBottomPanel((prev) => (prev === "composer" ? null : prev));
          }}
          onOpenScratchpad={() => {
            setActiveBottomPanel("scratchpad");
          }}
          onCloseScratchpad={() => {
            setActiveBottomPanel((prev) => (prev === "scratchpad" ? null : prev));
          }}
          onOpenSessionSettings={() => {
            setActiveBottomPanel("settings");
          }}
          onCloseSessionSettings={() => {
            setActiveBottomPanel((prev) => (prev === "settings" ? null : prev));
          }}
          onOpenContextDrawer={() => {
            setKnowledgeOpen(true);
            setHistoryOpen(false);
          }}
          onCloseContextDrawer={() => {
            setKnowledgeOpen(false);
          }}
          onPortalStepChange={setActionsMenuTourOpen}
          onQueueStepChange={setQueueTourOpen}
        />
        <ShellHeader
          shellData={shellData}
          activeUsers={activeUsers}
          portalId={portalId}
          sidebarOpen={sidebarOpen}
          onToggleSidebar={() => setSidebarOpen((prev) => !prev)}
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
          }}
          historyCount={shellTasks.length}
          activeQueueCount={activeQueuedTasks.length + activeRunningTasks.length}
          pivotCount={pivots.filter((p) => !p.closedAt).length}
          knowledgeOpen={knowledgeOpen}
          onToggleKnowledge={() => {
            setKnowledgeOpen((prev) => !prev);
            setHistoryOpen(false);
          }}
          credentialCount={hostCredentials.length || allCredentials.length}
          credentials={allCredentials}
          hosts={networkHosts}
          sshModalOpenExternal={sshModalOpenExternal}
          sshModalInitialTarget={sshModalInitialTarget}
          sshModalInitialCredentialId={sshModalInitialCredentialId}
          onCloseSshModalExternal={() => {
            setSshModalOpenExternal(false);
            setSshModalInitialTarget("");
            setSshModalInitialCredentialId(undefined);
          }}
          onLoadIntoComposer={handleLoadIntoComposer}
          composerOpen={composerOpen}
          onToggleComposer={() => {
            setActiveBottomPanel((prev) => (prev === "composer" ? null : "composer"));
          }}
          onSpawnFreshShell={handleSpawnFreshShell}
          onReplayTutorial={() => {
            setHistoryOpen(false);
            setKnowledgeOpen(false);
            setShowNuxBanner(true);
          }}
          actionsMenuTourOpen={actionsMenuTourOpen}
        />

        <InterpreterStatusBanner
          closedAt={shellData?.node?.closedAt}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          isConnecting={isConnecting}
          stuckTask={
            stuckTask && !completedTaskIdSet.has(String(stuckTask.id))
              ? { task: stuckTask, elapsedSeconds: stuckElapsedSeconds }
              : null
          }
          queuedTasks={activeQueuedTasks}
          runningTasks={activeRunningTasks}
          timeUntilCallback={timeUntilCallback}
          onSpawnFreshShell={handleSpawnFreshShell}
          isSpawningShell={isSpawningShell}
          onReconnect={handleReconnect}
        />

        {/* Main Workspace Row: Terminal/Pivots + Optional History or Guide Drawer */}
        <div className="flex-1 flex min-h-0 overflow-hidden">
          <ShellSidebar isOpen={sidebarOpen} />
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
                bg="#111827"
                borderBottomColor="#1f2937"
                display={useTabs ? "flex" : "none"}
                overflowX="auto"
                overflowY="hidden"
                className="bg-gray-900 border-b border-gray-800 gap-1 px-2 pt-1 pb-0"
              >
                <Tab
                  onMouseDown={(e) => e.preventDefault()}
                  _selected={{
                    color: "white",
                    bg: "#1f2937",
                    borderColor: "#374151",
                    borderBottomColor: "transparent",
                  }}
                  color="#9ca3af"
                  borderColor="transparent"
                  className="text-xs font-medium px-3 py-1.5 rounded-t-md"
                >
                  <span className="flex items-center gap-1.5">
                    <span
                      className={`w-2 h-2 rounded-full ${getMainTabIndicatorClass(
                        connectionStatus,
                        isConnecting
                      )}`}
                    />
                    <span>{shellData?.node?.beacon?.name ?? "Eldritch Shell"}</span>
                  </span>
                </Tab>

                {portalTabs.map((tab) => {
                  const status = tabStatuses[tab.id];
                  const isDisconnected = status === "disconnected";
                  const isTabConnecting = status === "connecting";
                  const isSsh = tab.type.toLowerCase() === "ssh";
                  const tabLabel = formatPivotTabLabel(tab);

                  return (
                    <Tab
                      key={tab.id}
                      onMouseDown={(e) => e.preventDefault()}
                      onContextMenu={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        setContextMenu({
                          isOpen: true,
                          x: e.clientX,
                          y: e.clientY,
                          tab,
                        });
                      }}
                      onDoubleClick={() => setRenameModalTab(tab)}
                      _selected={{
                        color: "white",
                        bg: "#1f2937",
                        borderColor: "#374151",
                        borderBottomColor: "transparent",
                      }}
                      color="#9ca3af"
                      borderColor="transparent"
                      className="text-xs font-medium px-3 py-1.5 rounded-t-md group"
                    >
                      <span className="flex items-center gap-1.5">
                        <PortalTabIcon
                          isDisconnected={isDisconnected}
                          isConnecting={isTabConnecting}
                          isSsh={isSsh}
                        />

                        <span className={isDisconnected ? "line-through text-gray-400" : ""}>
                          {tabLabel}
                        </span>

                        {isDisconnected && (
                          <span className="px-1.5 py-0.5 rounded bg-red-500/20 text-red-300 text-[10px] font-semibold">
                            Ended
                          </span>
                        )}

                        <span
                          role="button"
                          tabIndex={0}
                          aria-label={`Close ${tabLabel} tab`}
                          onClick={(e) => handleClosePortalTab(tab.id, e)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.preventDefault();
                              handleClosePortalTab(tab.id);
                            }
                          }}
                          className="ml-1 p-0.5 rounded hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
                        >
                          <Minus className="w-3.5 h-3.5" />
                        </span>
                      </span>
                    </Tab>
                  );
                })}
              </TabList>

              <TabPanels flex="1" display="flex" flexDirection="column" overflow="hidden">
                <TabPanel
                  flex="1"
                  p={0}
                  display="flex"
                  flexDirection="column"
                  overflow="hidden"
                  _hidden={{ display: "none" }}
                >
                  <ShellTerminal
                    termRef={termRef}
                    fontSize={shellFontSize}
                    isConnecting={isConnecting}
                    isPromptBusy={isPromptBusy}
                    isMissedCallback={isMissedCallback}
                    promptPos={promptPos}
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

                {portalTabs.map((tab, idx) => (
                  <TabPanel
                    key={tab.id}
                    flex="1"
                    p={0}
                    display="flex"
                    flexDirection="column"
                    overflow="hidden"
                    _hidden={{ display: "none" }}
                  >
                    {tab.type === "ssh" && (portalId || tab.pivotId) && (
                      <SshTerminal
                        portalId={portalId || 0}
                        target={tab.target}
                        credentialId={tab.credentialId}
                        pivotId={tab.pivotId ? parseInt(tab.pivotId, 10) : undefined}
                        shellId={shellId || ""}
                        isActive={tabIndex === idx + 1}
                        layoutKey={`${knowledgeOpen}:${historyOpen}:${composerOpen}:${sessionSettingsOpen}:${scratchpadOpen}:${shellFontSize}`}
                        fontSize={shellFontSize}
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
                        layoutKey={`${knowledgeOpen}:${historyOpen}:${composerOpen}:${sessionSettingsOpen}:${scratchpadOpen}:${shellFontSize}`}
                        fontSize={shellFontSize}
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
            <MultiLineComposer
              isOpen={composerOpen}
              onToggle={() =>
                setActiveBottomPanel((prev) => (prev === "composer" ? null : "composer"))
              }
              code={composerCode}
              onChange={setComposerCode}
              onRunCode={(code) => {
                setTabIndex(0);
                submitShellCode(code);
              }}
              disabled={!!shellData?.node?.closedAt}
              tomes={tomes}
              fontSize={shellFontSize}
              defaultAuthor={shellData?.node?.owner?.name}
              onTomeCreated={(createdTome) => {
                refetchTomes?.();
                toast({
                  title: "Tome created",
                  description: `"${createdTome.name}" has been saved and is now available in your tomes.`,
                  status: "success",
                  duration: 4000,
                  isClosable: true,
                });
              }}
            />
            <SessionSettings
              isOpen={sessionSettingsOpen}
              onToggle={() =>
                setActiveBottomPanel((prev) => (prev === "settings" ? null : "settings"))
              }
              onClose={() =>
                setActiveBottomPanel((prev) => (prev === "settings" ? null : prev))
              }
              disabled={!!shellData?.node?.closedAt}
              tomes={tomes}
              rcCode={rcCode}
              onRcCodeChange={setRcCode}
              rcEnabled={rcEnabled}
              onRcEnabledChange={setRcEnabled}
              fontSize={shellFontSize}
              onFontSizeChange={setShellFontSize}
            />
            <ShellScratchpad
              isOpen={scratchpadOpen}
              onToggle={() =>
                setActiveBottomPanel((prev) => (prev === "scratchpad" ? null : "scratchpad"))
              }
              shellId={shellId}
              fontSize={shellFontSize}
            />
          </div>

          <ShellKnowledgeDrawer
            isOpen={knowledgeOpen}
            onClose={() => setKnowledgeOpen(false)}
            currentShellId={shellId}
            currentHostId={shellData?.node?.beacon?.host?.id}
            currentHostName={shellData?.node?.beacon?.host?.name}
            currentHostIp={shellData?.node?.beacon?.host?.primaryIP}
            portalId={portalId}
            pivots={pivots.map((p) => {
              const matchingTab =
                portalTabs.find((t) => t.pivotId && String(t.pivotId) === String(p.id)) ||
                (!p.closedAt && String(p.kind || "").toLowerCase() === "pty"
                  ? portalTabs.find((t) => !t.pivotId && t.type === "pty")
                  : undefined);
              const activeTarget =
                (matchingTab?.target && matchingTab.target !== "PTY"
                  ? matchingTab.target
                  : undefined) ||
                pivotTargets[String(p.id)] ||
                matchingTab?.target ||
                (!p.closedAt && String(p.kind || "").toLowerCase() === "pty"
                  ? lastUnboundPtyTargetRef.current || undefined
                  : undefined);
              if (activeTarget) {
                return { ...p, activeTarget };
              }
              return p;
            })}
            hostCredentials={hostCredentials}
            allCredentials={allCredentials}
            networkHosts={networkHosts}
            hostFiles={hostFiles}
            hostProcessCount={hostProcessCount}
            hostShells={hostShells}
            lastKnowledgeSyncAt={lastKnowledgeSyncAt}
            onRefetchKnowledgeGraph={() => refetchKnowledgeGraph()}
            onNewPortal={handleNewPortal}
            onClosePortal={handleClosePortal}
            onOpenPty={handlePtyOpen}
            onOpenSshModal={() => {
              setSshModalInitialTarget("");
              setSshModalInitialCredentialId(undefined);
              setSshModalOpenExternal(true);
            }}
            onOpenPivotTab={(type, target, pivotId) =>
              handleOpenPortalTab(type, target, pivotId, true)
            }
            onOpenSshWithTarget={(target, credentialId) => {
              setSshModalInitialTarget(target);
              setSshModalInitialCredentialId(credentialId);
              setSshModalOpenExternal(true);
            }}
            onLoadIntoComposer={handleLoadIntoComposer}
          />

          <ShellHistoryDrawer
            isOpen={historyOpen}
            onClose={() => setHistoryOpen(false)}
            shellTasks={shellTasks}
            allShellTasks={allShellTasks}
            currentShellId={shellId}
            onRunCode={handleRunCodeFromDrawer}
            onLoadIntoComposer={handleLoadIntoComposer}
            onSelectShell={(targetShellId) => navigate(`/shells/${targetShellId}`)}
          />
        </div>

        <ShellStatusBar
          portalId={portalId}
          timeUntilCallback={timeUntilCallback}
          isMissedCallback={isMissedCallback}
          callbackStats={callbackStats}
          connectionStatus={connectionStatus}
          connectionMessage={connectionMessage}
          isConnecting={isConnecting}
          closedAt={shellData?.node?.closedAt}
          queuedCount={activeQueuedTasks.length}
          runningCount={activeRunningTasks.length}
          queuedTasks={activeQueuedTasks}
          runningTasks={activeRunningTasks}
          queueTourOpen={queueTourOpen}
          sessionSettingsOpen={sessionSettingsOpen}
          onToggleSessionSettings={() => {
            setActiveBottomPanel((prev) => (prev === "settings" ? null : "settings"));
          }}
          composerOpen={composerOpen}
          onToggleComposer={() => {
            setActiveBottomPanel((prev) => (prev === "composer" ? null : "composer"));
          }}
          scratchpadOpen={scratchpadOpen}
          onToggleScratchpad={() => {
            setActiveBottomPanel((prev) => (prev === "scratchpad" ? null : "scratchpad"));
          }}
          onNewPortal={handleNewPortal}
          onClosePortal={handleClosePortal}
          onReconnect={handleReconnect}
        />

        <TabContextMenu
          isOpen={contextMenu.isOpen}
          x={contextMenu.x}
          y={contextMenu.y}
          tabLabel={contextMenu.tab ? formatPivotTabLabel(contextMenu.tab) : ""}
          hasCustomName={Boolean(contextMenu.tab?.customTitle)}
          onClose={() => setContextMenu((prev) => ({ ...prev, isOpen: false }))}
          onRename={() => {
            if (contextMenu.tab) {
              setRenameModalTab(contextMenu.tab);
            }
          }}
          onResetName={() => {
            if (contextMenu.tab) {
              handleResetTabName(contextMenu.tab.id);
            }
          }}
          onCloseTab={() => {
            if (contextMenu.tab) {
              handleClosePortalTab(contextMenu.tab.id);
            }
          }}
        />

        <RenameTabModal
          isOpen={Boolean(renameModalTab)}
          onClose={() => setRenameModalTab(null)}
          currentName={renameModalTab ? formatPivotTabLabel(renameModalTab) : ""}
          tabType={renameModalTab?.type || "PTY"}
          hasCustomName={Boolean(renameModalTab?.customTitle)}
          onRename={(newName) => {
            if (renameModalTab) {
              handleRenameTab(renameModalTab.id, newName);
            }
          }}
          onResetName={() => {
            if (renameModalTab) {
              handleResetTabName(renameModalTab.id);
            }
          }}
        />
      </div>
  );
};

const Shell = () => (
  <AccessGate>
    <ShellContent />
  </AccessGate>
);

export default Shell;
