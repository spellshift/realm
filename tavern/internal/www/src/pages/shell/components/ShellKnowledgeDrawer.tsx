import React, { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useMutation } from "@apollo/client";
import {
  Check,
  Code2,
  Copy,
  Database,
  ExternalLink,
  Eye,
  EyeOff,
  FileText,
  KeyRound,
  Network,
  Plus,
  Radar,
  RefreshCw,
  Search,
  Server,
  Terminal,
  X,
} from "lucide-react";
import { CREATE_CREDENTIAL_MUTATION } from "../graphql";
import {
  KnowledgeCredentialItem,
  KnowledgeFileItem,
  KnowledgeHostItem,
  KnowledgeShellItem,
} from "../hooks/useShellData";

interface ShellKnowledgeDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  currentShellId?: string;
  currentHostId?: string;
  currentHostName?: string;
  currentHostIp?: string;
  portalId: number | null;
  hostCredentials: KnowledgeCredentialItem[];
  allCredentials: KnowledgeCredentialItem[];
  networkHosts: KnowledgeHostItem[];
  hostFiles: KnowledgeFileItem[];
  hostProcessCount: number;
  hostShells: KnowledgeShellItem[];
  lastKnowledgeSyncAt: number;
  onRefetchKnowledgeGraph: () => void;
  onOpenSshWithTarget: (initialTarget: string) => void;
  onLoadIntoComposer: (code: string) => void;
  onStageToPrompt: (code: string) => void;
}

const formatCredKind = (kind?: string) => {
  if (!kind) return "PASSWORD";
  return kind.replace(/^KIND_/, "");
};

const getSubnetCidr = (ip?: string | null): string => {
  if (!ip) return "10.0.0.0/24";
  const parts = ip.trim().split(".");
  if (parts.length !== 4) return ip;
  return `${parts[0]}.${parts[1]}.${parts[2]}.0/24`;
};

const ShellKnowledgeDrawer: React.FC<ShellKnowledgeDrawerProps> = ({
  isOpen,
  onClose,
  currentShellId,
  currentHostId,
  currentHostName,
  currentHostIp,
  portalId,
  hostCredentials,
  allCredentials,
  networkHosts,
  hostFiles,
  hostProcessCount,
  hostShells,
  lastKnowledgeSyncAt,
  onRefetchKnowledgeGraph,
  onOpenSshWithTarget,
  onLoadIntoComposer,
  onStageToPrompt,
}) => {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<"credentials" | "network" | "host">("credentials");
  const [credScope, setCredScope] = useState<"host" | "all">("host");
  const [searchQuery, setSearchQuery] = useState("");
  const [revealedIds, setRevealedIds] = useState<Set<string>>(new Set());
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  // Add credential inline form state
  const [showAddCred, setShowAddCred] = useState(false);
  const [newPrincipal, setNewPrincipal] = useState("");
  const [newSecret, setNewSecret] = useState("");
  const [newKind, setNewKind] = useState<"KIND_PASSWORD" | "KIND_SSH_KEY" | "KIND_NTLM_HASH">(
    "KIND_PASSWORD"
  );

  const [createCredential, { loading: isCreatingCred }] = useMutation(CREATE_CREDENTIAL_MUTATION);

  const toggleReveal = (id: string) => {
    setRevealedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const copyToClipboard = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey((curr) => (curr === key ? null : curr)), 1500);
  };

  const handleCreateCredential = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentHostId || !newPrincipal.trim() || !newSecret.trim()) return;
    try {
      await createCredential({
        variables: {
          input: {
            hostID: currentHostId,
            principal: newPrincipal.trim(),
            secret: newSecret,
            kind: newKind,
          },
        },
      });
      setNewPrincipal("");
      setNewSecret("");
      setShowAddCred(false);
      onRefetchKnowledgeGraph();
    } catch {
      // Ignore error or let user retry
    }
  };

  const displayedCredentials = useMemo(() => {
    const base = credScope === "host" ? hostCredentials : allCredentials;
    if (!searchQuery.trim()) return base;
    const q = searchQuery.toLowerCase();
    return base.filter(
      (c) =>
        c.principal.toLowerCase().includes(q) ||
        c.hostName.toLowerCase().includes(q) ||
        (c.hostPrimaryIP && c.hostPrimaryIP.toLowerCase().includes(q)) ||
        c.kind.toLowerCase().includes(q)
    );
  }, [credScope, hostCredentials, allCredentials, searchQuery]);

  const displayedHosts = useMemo(() => {
    if (!searchQuery.trim()) return networkHosts;
    const q = searchQuery.toLowerCase();
    return networkHosts.filter(
      (h) =>
        h.name.toLowerCase().includes(q) ||
        (h.primaryIP && h.primaryIP.toLowerCase().includes(q)) ||
        h.tags.some((t) => t.name.toLowerCase().includes(q))
    );
  }, [networkHosts, searchQuery]);

  if (!isOpen) return null;

  const syncTimeStr = new Date(lastKnowledgeSyncAt).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  return (
    <aside
      aria-label="Knowledge Graph Context"
      data-testid="shell-knowledge-drawer"
      className="w-full md:w-96 lg:w-[440px] flex-shrink-0 flex flex-col rounded-md bg-[#161920] border border-[#2e3440] h-full overflow-hidden shadow-xl"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 bg-[#1b1f2a] border-b border-[#2e3440]">
        <div className="flex items-center gap-2 min-w-0">
          <Database size={16} className="text-emerald-400 flex-shrink-0" />
          <h2 className="text-sm font-semibold text-white truncate">Knowledge Graph Intel</h2>
          <span
            title={`Auto-syncing every 3s (Last sync: ${syncTimeStr})`}
            className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-emerald-500/15 border border-emerald-500/30 text-[10px] text-emerald-300 font-medium"
          >
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            Live
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={onRefetchKnowledgeGraph}
            title={`Refresh Knowledge Graph (Last synced ${syncTimeStr})`}
            aria-label="Refresh knowledge graph"
            className="p-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] transition-colors"
          >
            <RefreshCw size={14} />
          </button>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close knowledge graph drawer"
            className="p-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] transition-colors"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      {/* Sub-tabs */}
      <div className="flex border-b border-[#2e3440] bg-[#181b24] text-xs">
        <button
          type="button"
          onClick={() => setActiveTab("credentials")}
          className={`flex-1 py-2 px-2.5 font-medium border-b-2 transition-colors ${
            activeTab === "credentials"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Credentials ({allCredentials.length})
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("network")}
          className={`flex-1 py-2 px-2.5 font-medium border-b-2 transition-colors ${
            activeTab === "network"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Network ({networkHosts.length})
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("host")}
          className={`flex-1 py-2 px-2.5 font-medium border-b-2 transition-colors ${
            activeTab === "host"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Host Context ({hostShells.length})
        </button>
      </div>

      {/* Search bar (for Credentials and Network tabs) */}
      {(activeTab === "credentials" || activeTab === "network") && (
        <div className="p-2.5 border-b border-[#2e3440] bg-[#161920] space-y-2">
          <div className="relative">
            <Search size={14} className="text-gray-500 absolute left-2.5 top-2.5" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={
                activeTab === "credentials"
                  ? "Filter by principal, host, IP, or type..."
                  : "Filter hosts by name, IP, or tag..."
              }
              className="w-full pl-8 pr-3 py-1.5 rounded bg-[#101217] border border-[#2e3440] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
            />
          </div>

          {activeTab === "credentials" && (
            <div className="flex items-center justify-between gap-2 text-[11px]">
              <div className="flex items-center gap-1">
                <button
                  type="button"
                  onClick={() => setCredScope("host")}
                  className={`px-2 py-1 rounded font-medium transition-colors ${
                    credScope === "host"
                      ? "bg-purple-600 text-white"
                      : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                  }`}
                >
                  This Host ({hostCredentials.length})
                </button>
                <button
                  type="button"
                  onClick={() => setCredScope("all")}
                  className={`px-2 py-1 rounded font-medium transition-colors ${
                    credScope === "all"
                      ? "bg-purple-600 text-white"
                      : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                  }`}
                >
                  All Realm Hosts ({allCredentials.length})
                </button>
              </div>

              {currentHostId && (
                <button
                  type="button"
                  onClick={() => setShowAddCred((prev) => !prev)}
                  className="inline-flex items-center gap-1 px-2 py-1 rounded bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-500/30 text-emerald-300 font-medium transition-colors"
                >
                  <Plus size={12} />
                  Record Cred
                </button>
              )}
            </div>
          )}
        </div>
      )}

      {/* TAB 1: CREDENTIALS */}
      {activeTab === "credentials" && (
        <div className="flex-1 overflow-y-auto p-2.5 space-y-2.5">
          {showAddCred && currentHostId && (
            <form
              onSubmit={handleCreateCredential}
              className="rounded-md bg-[#1c2230] border border-emerald-500/40 p-3 space-y-2 text-xs"
            >
              <div className="flex items-center justify-between text-emerald-300 font-semibold">
                <span>Record Credential on {currentHostName || "Host"}</span>
                <button
                  type="button"
                  onClick={() => setShowAddCred(false)}
                  className="text-gray-400 hover:text-white"
                >
                  <X size={13} />
                </button>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <input
                  type="text"
                  value={newPrincipal}
                  onChange={(e) => setNewPrincipal(e.target.value)}
                  placeholder="Principal (e.g. root)"
                  className="px-2 py-1.5 rounded bg-[#11141c] border border-[#2e3546] text-gray-200 font-mono text-xs"
                  required
                />
                <select
                  value={newKind}
                  onChange={(e) => setNewKind(e.target.value as any)}
                  className="px-2 py-1.5 rounded bg-[#11141c] border border-[#2e3546] text-gray-200 text-xs"
                >
                  <option value="KIND_PASSWORD">Password</option>
                  <option value="KIND_SSH_KEY">SSH Key</option>
                  <option value="KIND_NTLM_HASH">NTLM Hash</option>
                </select>
              </div>
              <input
                type="text"
                value={newSecret}
                onChange={(e) => setNewSecret(e.target.value)}
                placeholder="Secret / Password / Key"
                className="w-full px-2 py-1.5 rounded bg-[#11141c] border border-[#2e3546] text-gray-200 font-mono text-xs"
                required
              />
              <div className="flex justify-end gap-1.5">
                <button
                  type="button"
                  onClick={() => setShowAddCred(false)}
                  className="px-2.5 py-1 rounded text-gray-400 hover:text-white"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isCreatingCred || !newPrincipal.trim() || !newSecret.trim()}
                  className="px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-medium disabled:opacity-40"
                >
                  {isCreatingCred ? "Saving..." : "Save to Graph"}
                </button>
              </div>
            </form>
          )}

          {displayedCredentials.length === 0 ? (
            <div className="text-center py-8 px-4 text-xs text-gray-400 space-y-2">
              <KeyRound size={22} className="mx-auto text-gray-600" />
              <p className="font-medium text-gray-300">
                {credScope === "host"
                  ? "No credentials captured on this host yet."
                  : "No matching credentials in the Knowledge Graph."}
              </p>
              {credScope === "host" && allCredentials.length > 0 && (
                <button
                  type="button"
                  onClick={() => setCredScope("all")}
                  className="text-purple-400 hover:text-purple-300 underline font-medium"
                >
                  View {allCredentials.length} credentials from other hosts in Realm
                </button>
              )}
              <p className="text-gray-500 text-[11px]">
                Credentials reported via{" "}
                <code className="font-mono text-purple-300">report.user_password()</code> or{" "}
                <code className="font-mono text-purple-300">report.ssh_key()</code> appear here in real time.
              </p>
            </div>
          ) : (
            displayedCredentials.map((cred) => {
              const isRevealed = revealedIds.has(cred.id);
              const kindLabel = formatCredKind(cred.kind);
              const targetIp = cred.hostPrimaryIP || currentHostIp || "127.0.0.1";
              const sshTarget = `${cred.principal}:${cred.secret}@${targetIp}:22`;

              return (
                <div
                  key={cred.id}
                  className="rounded-md bg-[#1b1f29] border border-[#2b3140] p-2.5 text-xs space-y-2"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <KeyRound size={13} className="text-amber-400 flex-shrink-0" />
                      <span className="font-mono font-semibold text-white truncate">
                        {cred.principal}
                      </span>
                      <span className="px-1.5 py-0.2 rounded bg-[#12151c] border border-[#2a3040] text-gray-300 font-mono text-[10px]">
                        {kindLabel}
                      </span>
                    </div>

                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] font-medium truncate max-w-[140px] ${
                        cred.isCurrentHost
                          ? "bg-emerald-500/15 text-emerald-300 border border-emerald-500/30"
                          : "bg-blue-500/15 text-blue-300 border border-blue-500/30"
                      }`}
                      title={
                        cred.hostPrimaryIP
                          ? `${cred.hostName} (${cred.hostPrimaryIP})`
                          : cred.hostName
                      }
                    >
                      {cred.isCurrentHost ? "This Host" : cred.hostName}
                    </span>
                  </div>

                  {/* Secret Row */}
                  <div className="flex items-center justify-between gap-2 px-2 py-1.5 rounded bg-[#12151c] border border-[#252a38] font-mono text-[11px]">
                    <span className="text-gray-200 truncate select-all">
                      {isRevealed
                        ? cred.secret
                        : "•".repeat(Math.min(16, Math.max(8, cred.secret?.length || 8)))}
                    </span>
                    <div className="flex items-center gap-1 flex-shrink-0">
                      <button
                        type="button"
                        onClick={() => toggleReveal(cred.id)}
                        aria-label={isRevealed ? "Hide secret" : "Reveal secret"}
                        className="p-1 rounded text-gray-400 hover:text-white hover:bg-[#232836]"
                      >
                        {isRevealed ? <EyeOff size={12} /> : <Eye size={12} />}
                      </button>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(cred.secret, `sec-${cred.id}`)}
                        title="Copy secret"
                        className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-gray-300 hover:text-white hover:bg-[#232836]"
                      >
                        {copiedKey === `sec-${cred.id}` ? (
                          <Check size={11} className="text-emerald-400" />
                        ) : (
                          <Copy size={11} />
                        )}
                        <span>{copiedKey === `sec-${cred.id}` ? "Copied" : "Copy"}</span>
                      </button>
                    </div>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex flex-wrap items-center justify-end gap-1.5 pt-0.5">
                    <button
                      type="button"
                      onClick={() => {
                        const snippet =
                          kindLabel === "SSH_KEY"
                            ? `res = ssh.exec(\n    "${targetIp}",\n    "${cred.principal}",\n    "id && hostname",\n    key=${JSON.stringify(cred.secret)}\n)\nprint(res)`
                            : `res = ssh.exec(\n    "${targetIp}",\n    "${cred.principal}",\n    "id && hostname",\n    password=${JSON.stringify(cred.secret)}\n)\nprint(res)`;
                        onLoadIntoComposer(snippet);
                      }}
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 transition-colors"
                      title="Open an ssh.exec() snippet with this credential in the Multi-Line Composer"
                    >
                      <Code2 size={11} />
                      ssh.exec
                    </button>

                    <button
                      type="button"
                      onClick={() => {
                        const snippet = `creds = [{"principal": ${JSON.stringify(
                          cred.principal
                        )}, "password": ${JSON.stringify(
                          cred.secret
                        )}}]\nres = ssh.deploy(["${targetIp}"], creds, "id && hostname")\nprint(res)`;
                        onLoadIntoComposer(snippet);
                      }}
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 transition-colors"
                      title="Stage ssh.deploy() with this credential in the Multi-Line Composer"
                    >
                      <Network size={11} />
                      ssh.deploy
                    </button>

                    {kindLabel !== "SSH_KEY" && (
                      <button
                        type="button"
                        onClick={() => onOpenSshWithTarget(sshTarget)}
                        disabled={!portalId}
                        title={
                          portalId
                            ? `Open interactive SSH Pivot tab as ${cred.principal}@${targetIp}`
                            : "Open a Portal first (portal.create()) to launch interactive SSH pivot tabs"
                        }
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-purple-600/80 hover:bg-purple-600 disabled:opacity-40 disabled:cursor-not-allowed text-white font-medium transition-colors"
                      >
                        <Terminal size={11} />
                        SSH Pivot
                      </button>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {/* TAB 2: NETWORK & KNOWN HOSTS */}
      {activeTab === "network" && (
        <div className="flex-1 overflow-y-auto p-2.5 space-y-2.5">
          {/* Subnet Discovery Action Card */}
          <div className="rounded-md bg-[#1c2230] border border-[#2f384d] p-2.5 text-xs space-y-2">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-blue-300 flex items-center gap-1.5">
                <Radar size={13} />
                Discover Adjacent Hosts ({getSubnetCidr(currentHostIp)})
              </span>
            </div>
            <p className="text-[11px] text-gray-400">
              Scan the local subnet from this beacon to discover live hosts and open services:
            </p>
            <div className="flex flex-wrap gap-1.5">
              <button
                type="button"
                onClick={() =>
                  onLoadIntoComposer(
                    `arp_hosts = scan.arp(["${getSubnetCidr(currentHostIp)}"])\nprint(arp_hosts)`
                  )
                }
                className="inline-flex items-center gap-1 px-2 py-1 rounded bg-[#262e40] hover:bg-[#313b52] text-blue-200 font-mono text-[11px]"
              >
                scan.arp(&quot;{getSubnetCidr(currentHostIp)}&quot;)
              </button>
              <button
                type="button"
                onClick={() =>
                  onLoadIntoComposer(
                    `open_ports = scan.ports(["${getSubnetCidr(
                      currentHostIp
                    )}"], [22, 80, 443, 445, 3389])\nprint(open_ports)`
                  )
                }
                className="inline-flex items-center gap-1 px-2 py-1 rounded bg-[#262e40] hover:bg-[#313b52] text-purple-200 font-mono text-[11px]"
              >
                scan.ports(...)
              </button>
            </div>
          </div>

          {displayedHosts.map((host) => {
            const ip = host.primaryIP || host.name;
            const firstCred = host.credentials[0] || hostCredentials[0] || allCredentials[0];
            const prefilledSsh = firstCred
              ? `${firstCred.principal}:${firstCred.secret}@${ip}:22`
              : `root@${ip}:22`;

            return (
              <div
                key={host.id}
                className={`rounded-md border p-2.5 text-xs space-y-2 ${
                  host.isCurrentHost
                    ? "bg-[#1d2230] border-purple-500/40"
                    : "bg-[#1b1f29] border-[#2b3140]"
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-1.5 min-w-0">
                    <Server
                      size={13}
                      className={host.isCurrentHost ? "text-purple-400" : "text-blue-400"}
                    />
                    <Link
                      to={`/hosts/${host.id}`}
                      className="font-semibold text-white hover:text-blue-300 underline truncate"
                    >
                      {host.name}
                    </Link>
                    {host.primaryIP && (
                      <span className="font-mono text-gray-400 text-[11px]">
                        ({host.primaryIP})
                      </span>
                    )}
                  </div>

                  <div className="flex items-center gap-1 flex-shrink-0">
                    {host.isCurrentHost && (
                      <span className="px-1.5 py-0.2 rounded bg-purple-500/20 text-purple-300 text-[10px] font-medium">
                        Current Host
                      </span>
                    )}
                    {!host.isCurrentHost && host.sameSubnet && (
                      <span className="px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-300 text-[10px] font-medium">
                        Same Subnet
                      </span>
                    )}
                    {!host.isCurrentHost &&
                      host.sharedTags.map((tag) => (
                        <span
                          key={tag}
                          className="px-1.5 py-0.2 rounded bg-blue-500/15 text-blue-300 text-[10px]"
                        >
                          {tag}
                        </span>
                      ))}
                  </div>
                </div>

                <div className="flex items-center justify-between text-[11px] text-gray-400">
                  <div className="flex items-center gap-3">
                    <span>{host.beaconCount} beacons</span>
                    <span>•</span>
                    <span className={host.credentialCount > 0 ? "text-amber-300 font-medium" : ""}>
                      {host.credentialCount} credentials
                    </span>
                  </div>

                  <div className="flex items-center gap-1.5">
                    {host.primaryIP && (
                      <button
                        type="button"
                        onClick={() =>
                          onLoadIntoComposer(
                            `res = scan.ports(["${host.primaryIP}"], [22, 80, 443, 445, 3389, 8080])\nprint(res)`
                          )
                        }
                        className="px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 transition-colors"
                      >
                        Scan
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => onOpenSshWithTarget(prefilledSsh)}
                      disabled={!portalId}
                      title={
                        portalId
                          ? `SSH Pivot to ${ip}`
                          : "Requires an active portal (portal.create())"
                      }
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-purple-600/80 hover:bg-purple-600 disabled:opacity-40 text-white font-medium transition-colors"
                    >
                      <Terminal size={11} />
                      SSH
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* TAB 3: HOST CONTEXT (SHELLS & FILES) */}
      {activeTab === "host" && (
        <div className="flex-1 overflow-y-auto p-2.5 space-y-3 text-xs">
          {/* Shells on this Host */}
          <div className="space-y-1.5">
            <div className="flex items-center justify-between text-gray-300 font-semibold px-1">
              <span>Shell Sessions on {currentHostName || "Host"}</span>
              <span className="text-[11px] text-gray-500 font-normal">
                {hostShells.length} total
              </span>
            </div>

            {hostShells.length === 0 ? (
              <div className="rounded bg-[#1b1f29] border border-[#2b3140] p-3 text-gray-500 text-center">
                No shell sessions found.
              </div>
            ) : (
              hostShells.map((s) => {
                const isCurrent = String(s.id) === String(currentShellId);
                const isClosed = !!s.closedAt;
                return (
                  <div
                    key={s.id}
                    className={`rounded-md border p-2.5 flex items-center justify-between gap-2 ${
                      isCurrent
                        ? "bg-[#1f2435] border-purple-500/50"
                        : "bg-[#1b1f29] border-[#2b3140]"
                    }`}
                  >
                    <div className="min-w-0 space-y-0.5">
                      <div className="flex items-center gap-1.5">
                        <span className="font-mono font-semibold text-gray-200">
                          Shell #{s.id}
                        </span>
                        {isCurrent && (
                          <span className="px-1.5 py-0.2 rounded bg-purple-500/20 text-purple-300 text-[10px] font-medium">
                            Current
                          </span>
                        )}
                        <span
                          className={`px-1.5 py-0.2 rounded text-[10px] ${
                            isClosed
                              ? "bg-gray-700/60 text-gray-400"
                              : "bg-emerald-500/15 text-emerald-300"
                          }`}
                        >
                          {isClosed ? "Closed" : "Active"}
                        </span>
                      </div>
                      <div className="text-[11px] text-gray-400 truncate">
                        {s.beaconName}
                        {s.ownerName ? ` • ${s.ownerName}` : ""}
                      </div>
                    </div>

                    {!isCurrent && (
                      <button
                        type="button"
                        onClick={() => navigate(`/shells/${s.id}`)}
                        className="px-2.5 py-1 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 font-medium flex-shrink-0"
                      >
                        Switch
                      </button>
                    )}
                  </div>
                );
              })
            )}
          </div>

          {/* Reported Files on Host */}
          <div className="space-y-1.5 pt-2 border-t border-[#262b38]">
            <div className="flex items-center justify-between text-gray-300 font-semibold px-1">
              <span>Reported Host Files</span>
              <span className="text-[11px] text-gray-500 font-normal">
                {hostProcessCount} processes tracked
              </span>
            </div>

            {hostFiles.length === 0 ? (
              <div className="rounded bg-[#1b1f29] border border-[#2b3140] p-3 text-gray-500 text-center">
                No files reported via <code className="font-mono text-purple-300">report.file()</code>{" "}
                on this host yet.
              </div>
            ) : (
              hostFiles.map((f) => (
                <div
                  key={f.id}
                  className="rounded-md bg-[#1b1f29] border border-[#2b3140] p-2.5 space-y-1.5"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono text-gray-200 truncate flex items-center gap-1.5">
                      <FileText size={12} className="text-blue-400 flex-shrink-0" />
                      {f.path}
                    </span>
                    <button
                      type="button"
                      onClick={() =>
                        onStageToPrompt(`print(file.read(${JSON.stringify(f.path)}))`)
                      }
                      className="px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 text-[11px] flex-shrink-0"
                    >
                      file.read
                    </button>
                  </div>
                  <div className="text-[10px] text-gray-500 font-mono">
                    {f.permissions || ""} {f.owner ? `• ${f.owner}` : ""} • {f.size} bytes
                  </div>
                </div>
              ))
            )}

            {currentHostId && (
              <Link
                to={`/hosts/${currentHostId}`}
                className="flex items-center justify-between px-3 py-2 rounded-md bg-[#1e2330] hover:bg-[#262d3d] border border-[#2f374a] text-gray-200 font-medium transition-colors"
              >
                <span>Open Full Host Knowledge Graph Page</span>
                <ExternalLink size={13} />
              </Link>
            )}
          </div>
        </div>
      )}
    </aside>
  );
};

export default ShellKnowledgeDrawer;
