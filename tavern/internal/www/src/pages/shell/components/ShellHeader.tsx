import React from "react";
import { Tooltip } from "@chakra-ui/react";
import {
  BookOpen,
  Bug,
  Code2,
  Database,
  History,
  Home,
  Network,
  Server,
  Terminal,
  User,
} from "lucide-react";
import { Link } from "react-router-dom";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";
import { getEnumKey } from "../../../utils/utils";
import { SupportedPlatforms, SupportedTransports } from "../../../utils/enums";
import ShellActionsMenu from "./ShellActionsMenu";
import NotificationBell from "../../../components/notifications/NotificationBell";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";

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
    <div className="relative z-30 bg-[#181b24] px-3 py-1.5 rounded-md border border-[#262b38] flex flex-wrap justify-between items-center gap-2 text-xs font-mono text-gray-200 shadow-sm">
      {/* Left: Breadcrumbs & Context */}
      <div className="flex items-center gap-2 sm:gap-3 flex-wrap">
        <Link
          to="/"
          className="inline-flex items-center justify-center w-7 h-7 rounded-md bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border border-[#2f364a] transition-colors"
          title="Return Home"
        >
          <Home size={15} />
        </Link>

        {shellData?.node?.beacon?.host && (
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
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#1d222f] hover:bg-[#272e3f] border border-[#2f364a] text-purple-300 font-semibold transition-colors"
            >
              <Server size={13} />
              <span>{shellData.node.beacon.host.name}</span>
            </Link>
          </Tooltip>
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
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#1d222f] border border-[#2f364a] text-gray-200 cursor-default">
                <Terminal size={13} className="text-gray-400" />
                <span>{shellData.node.beacon.name}</span>
              </div>
            </Tooltip>

            {shellData.node.beacon.principal && (
              <>
                <span className="text-gray-600">/</span>
                <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#1d222f] border border-[#2f364a] text-gray-200 cursor-default">
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
        )}

        <div
          data-testid="shell-header-toolbar"
          className="flex items-center gap-0.5 p-1 rounded-lg bg-[#1d222f] border border-[#2f364a]"
        >
          {/* Composer Toggle */}
          {onToggleComposer && (
            <Tooltip
              label="Multi-line Script Composer (Alt+M)"
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <button
                type="button"
                data-tour="shell-composer-toggle"
                onClick={onToggleComposer}
                aria-label="Toggle multi-line composer"
                className={`${TOOLBAR_BUTTON_BASE} ${
                  composerOpen ? TOOLBAR_BUTTON_ACTIVE : TOOLBAR_BUTTON_INACTIVE
                }`}
              >
                <Code2 size={15} />
              </button>
            </Tooltip>
          )}

          {/* Knowledge Graph Drawer Toggle */}
          {onToggleKnowledge && (
            <Tooltip
              label="Knowledge Graph: Credentials, Hosts & Files"
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
                <Database size={15} />
                {credentialCount > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-purple-500 text-white text-[9px] font-bold flex items-center justify-center leading-none">
                    {credentialCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* Pivots Drawer Toggle (badge only for active sessions) */}
          {onTogglePivots && (
            <Tooltip
              label="Pivots (PTY & SSH Sessions)"
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
                <Network size={15} />
                {pivotCount > 0 && (
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-emerald-600 text-white text-[9px] font-bold flex items-center justify-center leading-none">
                    {pivotCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* History Drawer Toggle (badge only when tasks are actively queued or running) */}
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
                  <span className="absolute -top-1 -right-1 min-w-[15px] h-[15px] px-1 rounded-full bg-amber-400 text-black text-[9px] font-bold flex items-center justify-center leading-none">
                    {activeQueueCount > 99 ? "99+" : activeQueueCount}
                  </span>
                )}
              </button>
            </Tooltip>
          )}

          {/* Operator Guide & Eldritch Docs Toggle */}
          {onToggleNux && (
            <Tooltip
              label="Operator Guide & Eldritch v0.4.1 Reference"
              bg="#1f2430"
              color="white"
              borderRadius="md"
              border="1px solid #374151"
              hasArrow
            >
              <button
                type="button"
                data-tour="shell-guide"
                onClick={onToggleNux}
                aria-label="Toggle operator guide"
                className={`${TOOLBAR_BUTTON_BASE} ${
                  nuxOpen ? TOOLBAR_BUTTON_ACTIVE : TOOLBAR_BUTTON_INACTIVE
                }`}
              >
                <BookOpen size={15} />
              </button>
            </Tooltip>
          )}

          {/* Report a Bug */}
          <Tooltip
            label="Report a Bug"
            bg="#1f2430"
            color="white"
            borderRadius="md"
            border="1px solid #374151"
            hasArrow
          >
            <a
              data-tour="shell-bug-report"
              href="https://github.com/spellshift/realm/issues/new?template=bug_report.md"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Report a bug"
              className={`${TOOLBAR_BUTTON_BASE} ${TOOLBAR_BUTTON_INACTIVE}`}
            >
              <Bug size={15} />
            </a>
          </Tooltip>

          {/* Notifications */}
          <NotificationBell
            buttonClassName={`${TOOLBAR_BUTTON_BASE} ${TOOLBAR_BUTTON_INACTIVE}`}
            iconClassName="w-[15px] h-[15px]"
          />

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
