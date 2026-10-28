import React, { useState } from "react";
import { Link } from "react-router-dom";
import {
  BookOpen,
  Bug,
  ChevronRight,
  Code2,
  Database,
  Globe,
  History,
  Home,
  Server,
  SquareTerminal,
  Terminal,
} from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";
import { getEnumKey } from "../../../utils/utils";
import { SupportedPlatforms, SupportedTransports } from "../../../utils/enums";
import ShellActionsMenu from "./ShellActionsMenu";
import NotificationBell from "../../../components/notifications/NotificationBell";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";

interface ShellHeaderProps {
  shellData: any;
  activeUsers?: { id: string; name: string; photoURL?: string }[];
  portalId: number | null;
  onExport: () => void;
  onNewPortal: () => void;
  onClosePortal: () => void;
  onSshConnect: (target: string) => void;
  onPtyOpen: () => void;
  onSendCtrlC: () => void;
  onSendCtrlR: () => void;
  historyOpen?: boolean;
  onToggleHistory?: () => void;
  historyCount?: number;
  activeQueueCount?: number;
  knowledgeOpen?: boolean;
  onToggleKnowledge?: () => void;
  credentialCount?: number;
  credentials?: KnowledgeCredentialItem[];
  hosts?: KnowledgeHostItem[];
  sshModalOpenExternal?: boolean;
  sshModalInitialTarget?: string;
  onCloseSshModalExternal?: () => void;
  onLoadIntoComposer?: (code: string) => void;
  composerOpen?: boolean;
  onToggleComposer?: () => void;
  nuxOpen?: boolean;
  onToggleNux?: () => void;
  onSpawnFreshShell?: () => void;
}

const ShellHeader: React.FC<ShellHeaderProps> = ({
  shellData,
  activeUsers = [],
  portalId,
  onExport,
  onNewPortal,
  onClosePortal,
  onSshConnect,
  onPtyOpen,
  onSendCtrlC,
  onSendCtrlR,
  historyOpen = false,
  onToggleHistory,
  historyCount = 0,
  activeQueueCount = 0,
  knowledgeOpen = false,
  onToggleKnowledge,
  credentialCount = 0,
  credentials = [],
  hosts = [],
  sshModalOpenExternal,
  sshModalInitialTarget,
  onCloseSshModalExternal,
  onLoadIntoComposer,
  composerOpen = false,
  onToggleComposer,
  nuxOpen = false,
  onToggleNux,
  onSpawnFreshShell,
}) => {
  const [sshModalOpen, setSshModalOpen] = useState(false);

  const beaconName = shellData?.node?.beacon?.name;
  const principal = shellData?.node?.beacon?.principal;
  const agentIdentifier = shellData?.node?.beacon?.agentIdentifier;
  const interval = shellData?.node?.beacon?.interval;
  const transport = shellData?.node?.beacon?.transport;

  const hostName = shellData?.node?.beacon?.host?.name;
  const hostId = shellData?.node?.beacon?.host?.id;
  const primaryIP = shellData?.node?.beacon?.host?.primaryIP;
  const externalIP = shellData?.node?.beacon?.host?.externalIP;
  const platform = shellData?.node?.beacon?.host?.platform;
  const tags = shellData?.node?.beacon?.host?.tags?.edges?.map((edge: any) => edge.node) || [];

  return (
    <div className="flex flex-wrap items-center gap-2 md:gap-2.5 mb-2.5 pb-2 border-b border-[#252a36]">
      {/* Integrated Dark-Mode Breadcrumb & Target Identity */}
      <nav aria-label="Breadcrumb" className="flex items-center gap-1.5 min-w-0">
        <Link
          to="/"
          aria-label="Home"
          className="p-1.5 rounded-md text-gray-400 hover:text-gray-100 hover:bg-[#1e2330] transition-colors flex-shrink-0"
        >
          <Home size={15} />
        </Link>

        <ChevronRight size={14} className="text-gray-600 flex-shrink-0" />

        <Tooltip
          hasArrow
          label={
            <div className="flex flex-col text-xs">
              <span>Host: {hostName}</span>
              <span>Primary IP: {primaryIP || "N/A"}</span>
              {externalIP && <span>External IP: {externalIP}</span>}
              <span>Platform: {getEnumKey(SupportedPlatforms, platform)}</span>
              {tags.length > 0 && <span>Tags: {tags.map((t: any) => t.name).join(", ")}</span>}
            </div>
          }
        >
          <Link
            to={`/hosts/${hostId}`}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#181c27] hover:bg-[#212737] border border-[#293042] text-xs font-medium text-gray-100 hover:text-white transition-colors truncate max-w-[180px] sm:max-w-[240px]"
          >
            <Server size={13} className="text-blue-400 flex-shrink-0" />
            <span className="truncate">{hostName}</span>
            {primaryIP && (
              <span className="hidden xl:inline text-[11px] font-mono text-gray-400">
                ({primaryIP})
              </span>
            )}
          </Link>
        </Tooltip>

        <ChevronRight size={14} className="text-gray-600 flex-shrink-0 hidden sm:inline" />

        <Tooltip
          hasArrow
          label={
            <div className="flex flex-col text-xs">
              <span>Beacon: {beaconName}</span>
              <span>Principal: {principal}</span>
              <span>Agent Identifier: {agentIdentifier}</span>
              <span>Interval: {interval}s</span>
              <span>Transport: {getEnumKey(SupportedTransports, transport)}</span>
            </div>
          }
        >
          <div className="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-[#151822] border border-[#252b3b] text-xs text-gray-200 cursor-default">
            <Terminal size={13} className="text-purple-400 flex-shrink-0" />
            <span className="font-mono text-gray-200 truncate max-w-[160px] md:max-w-[220px]">
              {beaconName}
            </span>
            {principal && (
              <span className="px-1.5 py-0.2 rounded bg-purple-500/15 text-purple-300 border border-purple-500/30 text-[11px] font-medium">
                {principal}
              </span>
            )}
          </div>
        </Tooltip>
      </nav>

      {/* Mode & Pivot Quick Toolbar */}
      <div
        data-tour="shell-portal"
        className="hidden lg:flex items-center gap-1.5 ml-1 pl-2.5 border-l border-[#262b38]"
      >
        {portalId ? (
          <>
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              Portal #{portalId}
            </span>
            <button
              type="button"
              onClick={onPtyOpen}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border border-[#2f364a] transition-colors"
              title="Open interactive OS PTY tab over this portal"
            >
              <SquareTerminal size={13} className="text-emerald-400" />
              + PTY
            </button>
            <button
              type="button"
              onClick={() => setSshModalOpen(true)}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border border-[#2f364a] transition-colors"
              title="Open SSH pivot session through this portal"
            >
              <Terminal size={13} className="text-purple-400" />
              + SSH
            </button>
          </>
        ) : (
          <button
            type="button"
            onClick={onNewPortal}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 transition-colors"
            title="Upgrade from Queued Mode to an interactive low-latency Portal"
          >
            <Globe size={13} />
            Open Portal
          </button>
        )}
      </div>

      {/* Active Users & Icon-Only Workspace Toolbar */}
      <div className="ml-auto flex items-center gap-1.5 min-w-0">
        {activeUsers.length > 0 && (
          <div className="flex -space-x-2 mr-1">
            {activeUsers.map((user) => {
              const userImage =
                user.photoURL && user.photoURL !== "" ? user.photoURL : PlaceholderUser;
              return (
                <Tooltip key={user.id} label={user.name} hasArrow>
                  <img
                    src={userImage}
                    alt={user.name}
                    className="w-7 h-7 rounded-full border-2 border-[#161920] cursor-help object-cover"
                    title={user.name}
                  />
                </Tooltip>
              );
            })}
          </div>
        )}

        {/* Icon-Only Workspace Tool Buttons: Composer, Knowledge Graph, History, Guide */}
        {onToggleComposer && (
          <Tooltip label="Composer (Multi-Line Script Editor & Snippets)" hasArrow>
            <button
              type="button"
              data-tour="shell-composer"
              onClick={onToggleComposer}
              aria-label="Toggle multi-line composer"
              className={`inline-flex items-center justify-center p-1.5 rounded-md text-xs font-medium border transition-colors ${
                composerOpen
                  ? "bg-purple-600 text-white border-purple-500"
                  : "bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border-[#2f364a]"
              }`}
              title="Composer"
            >
              <Code2 size={15} />
            </button>
          </Tooltip>
        )}

        {onToggleKnowledge && (
          <Tooltip label="Knowledge Graph (Host & Network Credentials, Hosts, Reported Files)" hasArrow>
            <button
              type="button"
              data-tour="shell-knowledge"
              onClick={onToggleKnowledge}
              aria-label="Toggle knowledge graph"
              className={`inline-flex items-center gap-1 px-2 py-1.5 rounded-md text-xs font-medium border transition-colors ${
                knowledgeOpen
                  ? "bg-purple-600 text-white border-purple-500"
                  : "bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border-[#2f364a]"
              }`}
              title="Knowledge Graph"
            >
              <Database size={15} className={knowledgeOpen ? "text-white" : "text-emerald-400"} />
              {credentialCount > 0 && (
                <span
                  className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                    knowledgeOpen
                      ? "bg-purple-800 text-purple-100"
                      : "bg-emerald-500/20 text-emerald-300"
                  }`}
                >
                  {credentialCount}
                </span>
              )}
            </button>
          </Tooltip>
        )}

        {onToggleHistory && (
          <Tooltip label="Command History & Task Queue" hasArrow>
            <button
              type="button"
              data-tour="shell-history"
              onClick={onToggleHistory}
              aria-label="Toggle command history"
              className={`inline-flex items-center gap-1 px-2 py-1.5 rounded-md text-xs font-medium border transition-colors ${
                historyOpen
                  ? "bg-purple-600 text-white border-purple-500"
                  : "bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border-[#2f364a]"
              }`}
              title="Command History & Task Queue"
            >
              <History size={15} />
              {(historyCount > 0 || activeQueueCount > 0) && (
                <span
                  className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                    activeQueueCount > 0
                      ? "bg-amber-400 text-black font-bold"
                      : historyOpen
                      ? "bg-purple-800 text-purple-100"
                      : "bg-[#2c3345] text-gray-300"
                  }`}
                >
                  {activeQueueCount > 0 ? activeQueueCount : historyCount}
                </span>
              )}
            </button>
          </Tooltip>
        )}

        {onToggleNux && (
          <Tooltip label="Operator Guide & Eldritch Stdlib Reference" hasArrow>
            <button
              type="button"
              data-tour="shell-guide"
              onClick={onToggleNux}
              aria-label="Toggle operator guide"
              className={`inline-flex items-center justify-center p-1.5 rounded-md text-xs font-medium border transition-colors ${
                nuxOpen
                  ? "bg-purple-600 text-white border-purple-500"
                  : "bg-[#1d222f] hover:bg-[#272e3f] text-gray-200 border-[#2f364a]"
              }`}
              title="Operator Guide & Eldritch Reference"
            >
              <BookOpen size={15} />
            </button>
          </Tooltip>
        )}

        <a
          href="https://github.com/spellshift/realm/issues/new?template=bug_report.md&labels=bug&title=%5Bbug%5D%20Shell%3A%20%3CYOUR%20ISSUE%3E"
          target="_blank"
          rel="noopener noreferrer"
          className="hidden md:flex text-gray-400 hover:text-white transition-colors flex-shrink-0 p-1"
          title="Report a bug"
        >
          <Bug size={18} />
        </a>

        <div className="flex-shrink-0">
          <NotificationBell />
        </div>

        <div className="flex-shrink-0">
          <ShellActionsMenu
            portalId={portalId}
            onExport={onExport}
            onNewPortal={onNewPortal}
            onClosePortal={onClosePortal}
            onSshConnect={onSshConnect}
            onPtyOpen={onPtyOpen}
            onSendCtrlC={onSendCtrlC}
            onSendCtrlR={onSendCtrlR}
            onSpawnFreshShell={onSpawnFreshShell}
            onToggleHistory={onToggleHistory}
            onToggleKnowledge={onToggleKnowledge}
            onToggleNux={onToggleNux}
            onToggleComposer={onToggleComposer}
            sshModalOpenExternal={sshModalOpen || sshModalOpenExternal}
            sshModalInitialTarget={sshModalInitialTarget}
            onCloseSshModalExternal={() => {
              setSshModalOpen(false);
              onCloseSshModalExternal?.();
            }}
            credentials={credentials}
            hosts={hosts}
            onLoadIntoComposer={onLoadIntoComposer}
          />
        </div>
      </div>
    </div>
  );
};

export default ShellHeader;
