import React from "react";
import { Tooltip } from "@chakra-ui/react";
import { BugAntIcon } from "@heroicons/react/24/outline";
import {
  createLucideIcon,
  History,
  Home,
  Terminal,
  User,
} from "lucide-react";
import { Link } from "react-router-dom";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";
import { getEnumKey } from "../../../utils/utils";
import { SupportedPlatforms, SupportedTransports } from "../../../utils/enums";
import ShellActionsMenu from "./ShellActionsMenu";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";

export const GraphStar = createLucideIcon("graph-star", [
  ["path", { d: "m12.342 9.316 1.764-3.527", key: "p1" }],
  ["path", { d: "m12.59 14.545 2.35 3.759", key: "p2" }],
  ["path", { d: "m13.982 12.332 4.03.447", key: "p3" }],
  ["path", { d: "M5.414 17.586 8.88 14.12", key: "p4" }],
  ["path", { d: "m8.966 9.47-3.08-2.64", key: "p5" }],
  ["circle", { cx: "11", cy: "12", r: "3", key: "c0" }],
  ["circle", { cx: "15", cy: "4", r: "2", key: "c1" }],
  ["circle", { cx: "16", cy: "20", r: "2", key: "c2" }],
  ["circle", { cx: "20", cy: "13", r: "2", key: "c3" }],
  ["circle", { cx: "4", cy: "6", r: "2", key: "c4" }],
  ["circle", { cx: "4", cy: "19", r: "2", key: "c5" }],
]);

export const TOOLBAR_BUTTON_BASE =
  "relative inline-flex items-center justify-center w-7 h-7 rounded-md transition-colors";
export const TOOLBAR_BUTTON_INACTIVE =
  "bg-transparent hover:bg-[#282f42] text-gray-300 hover:text-white";
export const TOOLBAR_BUTTON_ACTIVE =
  "bg-purple-600 text-white shadow-sm";

interface ShellHeaderProps {
  shellData: any;
  activeUsers?: any[];
  portalId: number | null;
  onExport: () => void;
  onNewPortal: () => void;
  onSshConnect: (target: string) => void;
  onPtyOpen: () => void;
  onSendCtrlC: () => void;
  onSendCtrlR: () => void;
  onClosePortal: () => void;
  onSpawnFreshShell?: () => void;
  onReplayTutorial?: () => void;
  onToggleHistory?: () => void;
  historyOpen?: boolean;
  historyCount?: number;
  activeQueueCount?: number;
  onTogglePivots?: () => void;
  pivotsOpen?: boolean;
  pivotCount?: number;
  onToggleKnowledge?: () => void;
  knowledgeOpen?: boolean;
  credentialCount?: number;
  onToggleNux?: () => void;
  nuxOpen?: boolean;
  onToggleComposer?: () => void;
  composerOpen?: boolean;
  sshModalOpenExternal?: boolean;
  sshModalInitialTarget?: string;
  onCloseSshModalExternal?: () => void;
  credentials?: KnowledgeCredentialItem[];
  hosts?: KnowledgeHostItem[];
  onLoadIntoComposer?: (code: string) => void;
  actionsMenuTourOpen?: boolean;
}

const ShellHeader: React.FC<ShellHeaderProps> = ({
  shellData,
  activeUsers,
  portalId,
  onExport,
  onNewPortal,
  onSshConnect,
  onPtyOpen,
  onSendCtrlC,
  onSendCtrlR,
  onClosePortal,
  onSpawnFreshShell,
  onReplayTutorial,
  onToggleHistory,
  historyOpen = false,
  activeQueueCount = 0,
  onTogglePivots,
  pivotsOpen = false,
  pivotCount = 0,
  onToggleKnowledge,
  knowledgeOpen = false,
  credentialCount = 0,
  onToggleNux,
  nuxOpen = false,
  onToggleComposer,
  composerOpen = false,
  sshModalOpenExternal,
  sshModalInitialTarget,
  onCloseSshModalExternal,
  credentials,
  hosts,
  onLoadIntoComposer,
  actionsMenuTourOpen = false,
}) => {
  return (
    <div className="relative z-30 bg-[#181b24] px-3 py-1.5 border-b border-[#262b38] flex flex-wrap justify-between items-center gap-2 text-xs text-gray-200">
      {/* Left: Breadcrumbs & Context */}
      <div className="flex items-center gap-1.5 sm:gap-2 flex-wrap">
        <Link
          to="/"
          className="inline-flex items-center justify-center p-1.5 rounded-md hover:bg-[#272e3f] text-gray-300 hover:text-white transition-colors"
          title="Return Home"
        >
          <Home size={15} />
        </Link>

        {shellData?.node?.beacon?.host && (
          <>
            <span className="text-gray-600">/</span>
            <Tooltip
              label={
                <div className="flex flex-col text-xs p-1">
                  <span>Host: {shellData.node.beacon.host.name}</span>
                  <span>Primary IP: {shellData.node.beacon.host.primaryIP || "N/A"}</span>
                  {shellData.node.beacon.host.externalIP && (
                    <span>External IP: {shellData.node.beacon.host.externalIP}</span>
                  )}
                  <span>
                    Platform: {getEnumKey(SupportedPlatforms, shellData.node.beacon.host.platform)}
                  </span>
                </div>
              }
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <Link
                to={`/hosts/${shellData.node.beacon.host.id}`}
                className="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-[#272e3f] text-purple-300 font-semibold transition-colors"
              >
                <BugAntIcon className="w-3.5 h-3.5" />
                <span>{shellData.node.beacon.host.name}</span>
              </Link>
            </Tooltip>
          </>
        )}

        {shellData?.node?.beacon && (
          <>
            <span className="text-gray-600">/</span>
            <Tooltip
              label={
                <div className="flex flex-col text-xs p-1">
                  <span>Beacon: {shellData.node.beacon.name}</span>
                  <span>Agent: {shellData.node.beacon.agentIdentifier || "N/A"}</span>
                  <span>
                    Transport: {getEnumKey(SupportedTransports, shellData.node.beacon.transport)}
                  </span>
                </div>
              }
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <div className="flex items-center gap-1.5 px-2 py-1 rounded-md text-gray-200 cursor-default">
                <Terminal size={13} className="text-gray-400" />
                <span>{shellData.node.beacon.name}</span>
              </div>
            </Tooltip>

            {shellData.node.beacon.principal && (
              <>
                <span className="text-gray-600">/</span>
                <div className="flex items-center gap-1.5 px-2 py-1 rounded-md text-gray-200 cursor-default">
                  <User size={13} className="text-gray-400" />
                  <span>{shellData.node.beacon.principal}</span>
                </div>
              </>
            )}
          </>
        )}
      </div>

      {/* Right: Unified Action Toolbar */}
      <div className="flex items-center gap-2 ml-auto">
        {activeUsers && activeUsers.length > 0 && (
          <>
            <div className="flex -space-x-1.5 overflow-hidden">
              {activeUsers.map((user: any) => (
                <Tooltip key={user.id} label={user.name} hasArrow>
                  <img
                    className="inline-block h-6 w-6 rounded-full ring-2 ring-[#11131a]"
                    src={user.photoURL || PlaceholderUser}
                    alt={user.name}
                    onError={(e) => {
                      (e.target as HTMLImageElement).src = PlaceholderUser;
                    }}
                  />
                </Tooltip>
              ))}
            </div>
            <div
              data-testid="shell-header-users-divider"
              className="h-4 w-px bg-[#262b38]"
              aria-hidden="true"
            />
          </>
        )}

        <div
          data-testid="shell-header-toolbar"
          className="flex items-center gap-1"
        >
          {/* Context Drawer Toggle */}
          {onToggleKnowledge && (
            <Tooltip
              label="Context"
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <button
                type="button"
                data-tour="shell-knowledge"
                onClick={onToggleKnowledge}
                aria-label="Toggle knowledge graph"
                className={`${TOOLBAR_BUTTON_BASE} ${
                  knowledgeOpen ? TOOLBAR_BUTTON_ACTIVE : TOOLBAR_BUTTON_INACTIVE
                }`}
              >
                <GraphStar size={15} />
                {(pivotCount > 0 || credentialCount > 0) && (
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-purple-500 text-white text-[9px] font-bold flex items-center justify-center leading-none">
                    {pivotCount > 0 && !onTogglePivots ? pivotCount : credentialCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* Pivots Drawer Toggle (badge only for active sessions) */}
          {onTogglePivots && (
            <Tooltip
              label="Pivots"
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <button
                type="button"
                data-tour="shell-pivots"
                onClick={onTogglePivots}
                aria-label="Toggle pivots"
                className={`${TOOLBAR_BUTTON_BASE} ${
                  pivotsOpen ? TOOLBAR_BUTTON_ACTIVE : TOOLBAR_BUTTON_INACTIVE
                }`}
              >
                <GraphStar size={15} />
                {pivotCount > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-emerald-600 text-white text-[9px] font-bold flex items-center justify-center leading-none">
                    {pivotCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* History Drawer Toggle (badge only for active queued/executing tasks) */}
          {onToggleHistory && (
            <Tooltip
              label="History"
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <button
                type="button"
                data-tour="shell-history"
                onClick={onToggleHistory}
                aria-label="Toggle command history"
                className={`${TOOLBAR_BUTTON_BASE} ${
                  historyOpen ? TOOLBAR_BUTTON_ACTIVE : TOOLBAR_BUTTON_INACTIVE
                }`}
              >
                <History size={15} />
                {activeQueueCount > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-amber-500 text-gray-950 text-[9px] font-bold flex items-center justify-center leading-none">
                    {activeQueueCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* Shell Actions Menu */}
          <ShellActionsMenu
            portalId={portalId}
            onExport={onExport}
            onNewPortal={onNewPortal}
            onSshConnect={onSshConnect}
            onPtyOpen={onPtyOpen}
            onSendCtrlC={onSendCtrlC}
            onSendCtrlR={onSendCtrlR}
            onClosePortal={onClosePortal}
            onSpawnFreshShell={onSpawnFreshShell}
            onReplayTutorial={onReplayTutorial}
            sshModalOpenExternal={sshModalOpenExternal}
            sshModalInitialTarget={sshModalInitialTarget}
            onCloseSshModalExternal={onCloseSshModalExternal}
            credentials={credentials}
            hosts={hosts}
            onLoadIntoComposer={onLoadIntoComposer}
            buttonClassName={`${TOOLBAR_BUTTON_BASE} ${TOOLBAR_BUTTON_INACTIVE}`}
            forceOpen={actionsMenuTourOpen}
          />
        </div>
      </div>
    </div>
  );
};

export default ShellHeader;
