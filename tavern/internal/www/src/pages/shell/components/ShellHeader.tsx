import React, { useState } from "react";
import { Link } from "react-router-dom";
import { BookOpen, Bug, Code2, Database, Globe, History, SquareTerminal, Terminal } from "lucide-react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../../components/Breadcrumbs";
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
    <div className="flex flex-wrap items-center gap-2 md:gap-3 mb-3 pb-2.5 border-b border-[#2b303c]">
      <Breadcrumbs pages={[{ label: "Shell", link: window.location.pathname }]} />
      <h1 className="text-lg md:text-xl font-bold flex items-center gap-2 min-w-0">
        <span className="truncate max-w-[200px] md:max-w-none">
          <Tooltip
            label={
              <div className="flex flex-col">
                <span>Principal: {principal}</span>
                <span>Agent Identifier: {agentIdentifier}</span>
                <span>Interval: {interval}</span>
                <span>Transport: {getEnumKey(SupportedTransports, transport)}</span>
              </div>
            }
          >
            <span className="cursor-help border-b border-dashed border-gray-500 hidden md:inline">
              {beaconName}
            </span>
          </Tooltip>
          <span className="hidden md:inline">{" @ "}</span>
          <Tooltip
            label={
              <div className="flex flex-col">
                <span>Primary IP: {primaryIP}</span>
                <span>External IP: {externalIP}</span>
                <span>Platform: {getEnumKey(SupportedPlatforms, platform)}</span>
                {tags.length > 0 && <span>Tags: {tags.map((t: any) => t.name).join(", ")}</span>}
              </div>
            }
          >
            <Link to={`/hosts/${hostId}`} className="text-blue-400 hover:text-blue-300 underline">
              {hostName}
            </Link>
          </Tooltip>
        </span>
        {principal && (
          <span className="hidden md:inline">
            <Badge>{principal}</Badge>
          </span>
        )}
      </h1>

      {/* Mode & Pivot Quick Toolbar */}
      <div className="hidden lg:flex items-center gap-1.5 ml-2 pl-3 border-l border-[#2e3442]">
        {portalId ? (
          <>
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              Interactive Portal #{portalId}
            </span>
            <button
              type="button"
              onClick={onPtyOpen}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs font-medium bg-[#222734] hover:bg-[#2c3344] text-gray-200 border border-[#343c4f] transition-colors"
              title="Open interactive OS PTY tab over this portal"
            >
              <SquareTerminal size={13} className="text-emerald-400" />
              + PTY
            </button>
            <button
              type="button"
              onClick={() => setSshModalOpen(true)}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs font-medium bg-[#222734] hover:bg-[#2c3344] text-gray-200 border border-[#343c4f] transition-colors"
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
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 transition-colors"
            title="Upgrade from Queued Mode to an interactive low-latency Portal"
          >
            <Globe size={13} />
            Open Portal
          </button>
        )}
      </div>

      {/* Active Users Display */}
      <div className="ml-auto flex items-center gap-2 min-w-0">
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

        {/* Workspace Tool Buttons: Composer, Intel, History, Guide */}
        {onToggleComposer && (
          <button
            type="button"
            onClick={onToggleComposer}
            aria-label="Toggle multi-line composer"
            className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium border transition-colors ${
              composerOpen
                ? "bg-purple-600 text-white border-purple-500"
                : "bg-[#222734] hover:bg-[#2c3344] text-gray-200 border-[#343c4f]"
            }`}
            title="Multi-Line Eldritch Composer"
          >
            <Code2 size={14} />
            <span className="hidden sm:inline">Composer</span>
          </button>
        )}

        {onToggleKnowledge && (
          <button
            type="button"
            onClick={onToggleKnowledge}
            aria-label="Toggle knowledge graph"
            className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium border transition-colors ${
              knowledgeOpen
                ? "bg-purple-600 text-white border-purple-500"
                : "bg-[#222734] hover:bg-[#2c3344] text-gray-200 border-[#343c4f]"
            }`}
            title="Knowledge Graph Intel (Credentials, Hosts, Context)"
          >
            <Database size={14} className={knowledgeOpen ? "text-white" : "text-emerald-400"} />
            <span className="hidden sm:inline">Intel</span>
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
        )}

        {onToggleHistory && (
          <button
            type="button"
            onClick={onToggleHistory}
            aria-label="Toggle command history"
            className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium border transition-colors ${
              historyOpen
                ? "bg-purple-600 text-white border-purple-500"
                : "bg-[#222734] hover:bg-[#2c3344] text-gray-200 border-[#343c4f]"
            }`}
            title="Command History & Task Queue"
          >
            <History size={14} />
            <span className="hidden sm:inline">History</span>
            {(historyCount > 0 || activeQueueCount > 0) && (
              <span
                className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                  activeQueueCount > 0
                    ? "bg-amber-400 text-black font-bold"
                    : historyOpen
                    ? "bg-purple-800 text-purple-100"
                    : "bg-[#31384a] text-gray-300"
                }`}
              >
                {activeQueueCount > 0 ? `${activeQueueCount} queued` : historyCount}
              </span>
            )}
          </button>
        )}

        {onToggleNux && (
          <button
            type="button"
            onClick={onToggleNux}
            aria-label="Toggle operator guide"
            className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium border transition-colors ${
              nuxOpen
                ? "bg-purple-600 text-white border-purple-500"
                : "bg-[#222734] hover:bg-[#2c3344] text-gray-200 border-[#343c4f]"
            }`}
            title="Operator Guide & Eldritch Reference"
          >
            <BookOpen size={14} />
            <span className="hidden md:inline">Guide</span>
          </button>
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
