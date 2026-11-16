import React, { Fragment, useContext, useEffect, useMemo, useState } from "react";
import { getApolloContext } from "@apollo/client";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import { Check, Code2, CornerDownRight, KeyRound, Monitor, Plus, X } from "lucide-react";
import { CREATE_CREDENTIAL_MUTATION } from "../graphql";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";
import { formatSubnetCidr } from "../utils/networkGraphUtils";

export type PivotType = "SSH";

const PIVOT_TYPE_OPTIONS: { value: PivotType; label: string }[] = [
    { value: "SSH", label: "SSH" },
];

export interface PivotModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConnect: (target: string, credentialId?: string, pivotType?: PivotType) => void;
    initialTarget?: string;
    initialCredentialId?: string;
    credentials?: KnowledgeCredentialItem[];
    hosts?: KnowledgeHostItem[];
    currentHostId?: string;
    onCredentialCreated?: (cred: KnowledgeCredentialItem, targetHostId?: string) => void;
    onLoadIntoComposer?: (code: string) => void;
}

type SshConnectionModalProps = PivotModalProps;

const CREDENTIAL_KIND_OPTIONS = [
    { value: "KIND_PASSWORD", label: "Password" },
    { value: "KIND_SSH_KEY", label: "SSH Key" },
];

const formatCredKind = (kind?: string) => {
    if (!kind) return "CRED";
    return kind.replace(/^KIND_/, "");
};

const isSshKeyCredential = (cred?: KnowledgeCredentialItem): boolean => {
    if (!cred) return false;
    if (cred.kind?.toUpperCase().includes("SSH_KEY")) return true;
    return Boolean(cred.secret?.trim().startsWith("-----BEGIN"));
};

const parseSshTarget = (raw: string): { username: string; password: string; hostPort: string } => {
    const trimmed = raw.trim();
    if (!trimmed) {
        return { username: "", password: "", hostPort: "" };
    }
    const atIdx = trimmed.lastIndexOf("@");
    if (atIdx === -1) {
        return { username: "", password: "", hostPort: trimmed };
    }
    const userPass = trimmed.slice(0, atIdx);
    const hostPort = trimmed.slice(atIdx + 1);
    const colonIdx = userPass.indexOf(":");
    if (colonIdx === -1) {
        return { username: userPass, password: "", hostPort };
    }
    return {
        username: userPass.slice(0, colonIdx),
        password: userPass.slice(colonIdx + 1),
        hostPort,
    };
};

const splitHostAndPort = (hostPort: string): { host: string; port: string } => {
    const trimmed = hostPort.trim();
    if (!trimmed) {
        return { host: "", port: "" };
    }
    if (trimmed.startsWith("[")) {
        const bracketEnd = trimmed.indexOf("]");
        if (bracketEnd !== -1) {
            const after = trimmed.slice(bracketEnd + 1);
            if (after.startsWith(":")) {
                return { host: trimmed.slice(0, bracketEnd + 1), port: after.slice(1) };
            }
            return { host: trimmed, port: "" };
        }
    }
    const lastColon = trimmed.lastIndexOf(":");
    if (lastColon > 0 && trimmed.indexOf(":") === lastColon) {
        return {
            host: trimmed.slice(0, lastColon),
            port: trimmed.slice(lastColon + 1),
        };
    }
    return { host: trimmed, port: "" };
};

const buildSshTarget = (username: string, password: string, hostPort: string): string => {
    const hp = hostPort.trim();
    const user = username.trim();
    if (!user && !password) {
        return hp;
    }
    const effectiveUser = user || "root";
    const authPart = password ? `${effectiveUser}:${password}` : effectiveUser;
    if (!hp) {
        return `${authPart}@`;
    }
    return `${authPart}@${hp}`;
};

const doesCredMatchHost = (cred: KnowledgeCredentialItem, host: KnowledgeHostItem): boolean => {
    if (host.hostId && cred.hostId && String(cred.hostId) === String(host.hostId)) {
        return true;
    }
    if (cred.hostId && String(cred.hostId) === String(host.id)) {
        return true;
    }
    if ((host.credentials || []).some((hc) => String(hc.id) === String(cred.id))) {
        return true;
    }
    const hostIps = new Set(
        [host.ipAddress, host.primaryIP, host.externalIP]
            .filter(Boolean)
            .map((ip) => String(ip).trim())
    );
    if (cred.hostPrimaryIP && hostIps.has(String(cred.hostPrimaryIP).trim())) {
        return true;
    }
    const hostNames = new Set(
        [host.name, host.hostname]
            .filter(Boolean)
            .map((n) => String(n).trim().toLowerCase())
    );
    if (cred.hostName && hostNames.has(String(cred.hostName).trim().toLowerCase())) {
        return true;
    }
    return false;
};

const AvailablePivotsPicker: React.FC<{
    hosts: KnowledgeHostItem[];
    selectedHostId: string;
    onSelectHost: (host: KnowledgeHostItem) => void;
}> = ({ hosts, selectedHostId, onSelectHost }) => {
    if (hosts.length === 0) {
        return null;
    }

    return (
        <div className="space-y-1.5" data-testid="pivot-modal-available-pivots">
            <div className="text-[11px] text-gray-400 flex items-center gap-1">
                <Monitor size={11} className="text-purple-400" />
                <span>Available Pivots ({hosts.length})</span>
            </div>
            <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
                {hosts.map((h) => {
                    const isSelected = selectedHostId === h.id;
                    const rowClass = isSelected
                        ? "bg-purple-600/25 border-purple-400 text-purple-100"
                        : "bg-[#1d222e] hover:bg-[#262c3c] border-[#2f3647] text-gray-300";
                    const ip = h.ipAddress || h.primaryIP || "";
                    const subnet = h.subnet || formatSubnetCidr(ip);
                    const subMeta = [ip, subnet].filter(Boolean).join(" · ");

                    return (
                        <button
                            key={h.id}
                            type="button"
                            aria-pressed={isSelected}
                            data-testid={`ssh-host-option-${h.id}`}
                            onClick={() => onSelectHost(h)}
                            className={`flex items-center justify-between gap-2 px-2.5 py-1.5 rounded text-xs border text-left transition-colors ${rowClass}`}
                        >
                            <span className="flex items-center gap-1.5 min-w-0">
                                <Monitor className="w-3.5 h-3.5 shrink-0 text-gray-400" />
                                <span className="font-mono font-medium truncate">{h.name}</span>
                                {subMeta && (
                                    <span className="text-[11px] font-mono text-gray-400 truncate">
                                        ({subMeta})
                                    </span>
                                )}
                            </span>
                            <span className="flex items-center gap-1 shrink-0">
                                {(h.openPorts || []).slice(0, 3).map((port) => (
                                    <span
                                        key={port}
                                        className="px-1 py-0.5 rounded bg-[#12151c] text-gray-400 text-[10px] font-mono"
                                    >
                                        :{port}
                                    </span>
                                ))}
                                {h.isCurrentHost && (
                                    <span className="px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-300 text-[10px]">
                                        this host
                                    </span>
                                )}
                            </span>
                        </button>
                    );
                })}
            </div>
        </div>
    );
};

const CredentialPickerList: React.FC<{
    credentials: KnowledgeCredentialItem[];
    selectedCredId: string;
    onSelectCred: (cred: KnowledgeCredentialItem) => void;
    filteredHostName?: string;
}> = ({ credentials, selectedCredId, onSelectCred, filteredHostName }) => {
    if (credentials.length === 0) {
        return (
            <div className="text-xs text-gray-500 py-1">
                {filteredHostName
                    ? `No credentials associated with ${filteredHostName}. Use + to add one.`
                    : "No credentials available yet. Use + to add one."}
            </div>
        );
    }

    return (
        <div
            className="flex flex-col gap-1 max-h-40 overflow-y-auto"
            data-testid="ssh-credential-selector"
        >
            {credentials.map((c) => {
                const isSelected = selectedCredId === c.id;
                const rowClass = isSelected
                    ? "bg-purple-600/30 border-purple-400 text-purple-100"
                    : "bg-[#1d222e] hover:bg-[#262c3c] border-[#2f3647] text-gray-200";
                const hostBadgeClass = c.isCurrentHost
                    ? "bg-emerald-500/20 text-emerald-300"
                    : "bg-gray-700/60 text-gray-300";
                const maskPreview = isSshKeyCredential(c)
                    ? "SSH key"
                    : "*".repeat(Math.min(8, Math.max(4, c.secret?.length || 6)));

                return (
                    <button
                        key={c.id}
                        type="button"
                        aria-pressed={isSelected}
                        data-testid={`ssh-cred-option-${c.id}`}
                        onClick={() => onSelectCred(c)}
                        className={`flex items-center justify-between px-2.5 py-1.5 rounded text-xs border text-left transition-colors ${rowClass}`}
                    >
                        <span className="font-mono font-medium truncate">
                            {c.principal}
                            <span className="text-gray-500 ml-1.5">
                                ({maskPreview})
                            </span>
                        </span>
                        <span className="flex items-center gap-1.5 flex-shrink-0">
                            <span className="px-1.5 py-0.5 rounded bg-[#12151c] text-gray-400 text-[10px] font-mono">
                                {formatCredKind(c.kind)}
                            </span>
                            {c.hostName && (
                                <span className={`px-1.5 py-0.5 rounded text-[10px] ${hostBadgeClass}`}>
                                    {c.hostName}
                                </span>
                            )}
                        </span>
                    </button>
                );
            })}
        </div>
    );
};

interface InlineCredentialEditorProps {
    principal: string;
    kind: string;
    secret: string;
    isSaving: boolean;
    error: string | null;
    onPrincipalChange: (val: string) => void;
    onKindChange: (val: string) => void;
    onSecretChange: (val: string) => void;
    onSave: () => void;
    onCancel: () => void;
}

const InlineCredentialEditor: React.FC<InlineCredentialEditorProps> = ({
    principal,
    kind,
    secret,
    isSaving,
    error,
    onPrincipalChange,
    onKindChange,
    onSecretChange,
    onSave,
    onCancel,
}) => {
    return (
        <div
            data-testid="ssh-inline-credential-editor"
            className="rounded-md bg-[#12151c] border border-purple-500/40 p-2.5 space-y-2"
        >
            <div className="grid grid-cols-2 gap-2">
                <input
                    type="text"
                    aria-label="New credential principal"
                    placeholder="Principal (e.g. root)"
                    value={principal}
                    onChange={(e) => onPrincipalChange(e.target.value)}
                    className="px-2 py-1 rounded bg-[#1a1f2b] border border-[#2f3647] text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                />
                <select
                    aria-label="New credential kind"
                    value={kind}
                    onChange={(e) => onKindChange(e.target.value)}
                    className="px-2 py-1 rounded bg-[#1a1f2b] border border-[#2f3647] text-xs text-gray-200 focus:outline-none focus:border-purple-500"
                >
                    {CREDENTIAL_KIND_OPTIONS.map((opt) => (
                        <option key={opt.value} value={opt.value}>
                            {opt.label}
                        </option>
                    ))}
                </select>
            </div>
            <input
                type="text"
                aria-label="New credential secret"
                placeholder={kind === "KIND_SSH_KEY" ? "-----BEGIN OPENSSH PRIVATE KEY-----" : "Password or secret"}
                value={secret}
                onChange={(e) => onSecretChange(e.target.value)}
                className="w-full px-2 py-1 rounded bg-[#1a1f2b] border border-[#2f3647] text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
            />
            {error && (
                <div className="text-[11px] text-red-400">{error}</div>
            )}
            <div className="flex items-center justify-end gap-1.5">
                <button
                    type="button"
                    onClick={onCancel}
                    className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs text-gray-400 hover:text-gray-200 hover:bg-gray-800 transition-colors"
                >
                    <X className="w-3 h-3" />
                    Cancel
                </button>
                <button
                    type="button"
                    data-testid="ssh-save-inline-credential"
                    disabled={!principal.trim() || !secret.trim() || isSaving}
                    onClick={onSave}
                    className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-purple-700 hover:bg-purple-600 disabled:opacity-40 disabled:cursor-not-allowed text-xs font-medium text-white transition-colors"
                >
                    <Check className="w-3 h-3" />
                    Save
                </button>
            </div>
        </div>
    );
};

const SshConnectionModal: React.FC<SshConnectionModalProps> = ({
    isOpen,
    onClose,
    onConnect,
    initialTarget = "",
    initialCredentialId = "",
    credentials = [],
    hosts = [],
    currentHostId,
    onCredentialCreated,
    onLoadIntoComposer,
}) => {
    const apolloClient = useContext(getApolloContext())?.client;

    const [pivotType, setPivotType] = useState<PivotType>("SSH");
    const [target, setTarget] = useState(initialTarget);
    const [username, setUsername] = useState<string>("");
    const [password, setPassword] = useState<string>("");
    const [hostPort, setHostPort] = useState<string>("");
    const [selectedHostId, setSelectedHostId] = useState<string>("");
    const [selectedHostIp, setSelectedHostIp] = useState<string>("");
    const [selectedCredId, setSelectedCredId] = useState<string>(initialCredentialId);
    const [localCredentials, setLocalCredentials] = useState<KnowledgeCredentialItem[]>([]);

    const [isAddingCred, setIsAddingCred] = useState(false);
    const [newCredPrincipal, setNewCredPrincipal] = useState("");
    const [newCredKind, setNewCredKind] = useState("KIND_PASSWORD");
    const [newCredSecret, setNewCredSecret] = useState("");
    const [isSavingCred, setIsSavingCred] = useState(false);
    const [credSaveError, setCredSaveError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) {
            return;
        }
        const parsed = parseSshTarget(initialTarget);
        const { host } = splitHostAndPort(parsed.hostPort);
        setPivotType("SSH");
        setTarget(initialTarget);
        setUsername(parsed.username);
        setPassword(parsed.password);
        setHostPort(parsed.hostPort);
        setSelectedCredId(initialCredentialId || "");
        setSelectedHostIp(host);
        setSelectedHostId("");
        setIsAddingCred(false);
        setNewCredPrincipal("");
        setNewCredKind("KIND_PASSWORD");
        setNewCredSecret("");
        setCredSaveError(null);
    }, [isOpen, initialTarget, initialCredentialId]);

    const selectedHost = useMemo(
        () => hosts.find((h) => h.id === selectedHostId) || null,
        [hosts, selectedHostId]
    );

    const allCombinedCreds = useMemo(() => {
        const byId = new Map<string, KnowledgeCredentialItem>();
        for (const c of credentials) {
            if (c?.id) {
                byId.set(String(c.id), c);
            }
        }
        for (const h of hosts) {
            for (const c of h.credentials || []) {
                if (c?.id && !byId.has(String(c.id))) {
                    byId.set(String(c.id), c);
                }
            }
        }
        for (const c of localCredentials) {
            if (c?.id) {
                byId.set(String(c.id), c);
            }
        }
        return Array.from(byId.values());
    }, [credentials, hosts, localCredentials]);

    const relevantCreds = useMemo(() => {
        const filtered = selectedHost
            ? allCombinedCreds.filter((c) => doesCredMatchHost(c, selectedHost))
            : allCombinedCreds;

        const activeHost = selectedHostIp || splitHostAndPort(hostPort).host || "";
        return [...filtered].sort((a, b) => {
            const aMatchesHost = Boolean(
                activeHost && (a.hostPrimaryIP === activeHost || a.hostName === activeHost)
            );
            const bMatchesHost = Boolean(
                activeHost && (b.hostPrimaryIP === activeHost || b.hostName === activeHost)
            );
            if (aMatchesHost !== bMatchesHost) {
                return aMatchesHost ? -1 : 1;
            }
            if (Boolean(a.isCurrentHost) !== Boolean(b.isCurrentHost)) {
                return a.isCurrentHost ? -1 : 1;
            }
            return (a.principal || "").localeCompare(b.principal || "");
        });
    }, [allCombinedCreds, selectedHost, selectedHostIp, hostPort]);

    const handleTargetChange = (nextTarget: string) => {
        setTarget(nextTarget);
        const parsed = parseSshTarget(nextTarget);
        setUsername(parsed.username);
        setPassword(parsed.password);
        setHostPort(parsed.hostPort);
        const { host } = splitHostAndPort(parsed.hostPort);
        setSelectedHostIp(host);
    };

    const currentExplicitPort = (): string => {
        const { port } = splitHostAndPort(hostPort);
        return port || "22";
    };

    const applySelection = (credId: string, hostIp: string, credsList = allCombinedCreds) => {
        const cred = credsList.find((c) => c.id === credId);
        const fallbackHost = splitHostAndPort(hostPort).host || "";
        const ip = hostIp || fallbackHost || "127.0.0.1";
        const port = currentExplicitPort();
        const nextHostPort = `${ip}:${port}`;
        setHostPort(nextHostPort);
        if (cred) {
            setUsername(cred.principal);
            setPassword("");
            setTarget(buildSshTarget(cred.principal, "", nextHostPort));
        } else if (hostIp || fallbackHost) {
            setTarget(buildSshTarget(username || "root", password, nextHostPort));
        }
    };

    const handleSelectCred = (cred: KnowledgeCredentialItem) => {
        if (selectedCredId === cred.id) {
            setSelectedCredId("");
            const fallbackHost = selectedHostIp || splitHostAndPort(hostPort).host || "";
            if (fallbackHost) {
                const port = currentExplicitPort();
                setTarget(
                    buildSshTarget(username || cred.principal || "root", "", `${fallbackHost}:${port}`)
                );
            }
            return;
        }
        setSelectedCredId(cred.id);
        const ip = selectedHostIp || cred.hostPrimaryIP || "127.0.0.1";
        if (!selectedHostIp && cred.hostPrimaryIP) {
            setSelectedHostIp(cred.hostPrimaryIP);
        }
        applySelection(cred.id, ip);
    };

    const handleSelectHost = (host: KnowledgeHostItem) => {
        if (selectedHostId === host.id) {
            setSelectedHostId("");
            return;
        }
        setSelectedHostId(host.id);
        const ip = host.ipAddress || host.primaryIP || host.name;
        setSelectedHostIp(ip);

        const activeCred = allCombinedCreds.find((c) => c.id === selectedCredId);
        const keepCredId = activeCred && doesCredMatchHost(activeCred, host) ? selectedCredId : "";
        if (keepCredId !== selectedCredId) {
            setSelectedCredId(keepCredId);
        }
        applySelection(keepCredId, ip);
    };

    const handleSaveInlineCredential = async () => {
        const cleanPrincipal = newCredPrincipal.trim();
        if (!cleanPrincipal || !newCredSecret.trim() || isSavingCred) {
            return;
        }

        setIsSavingCred(true);
        setCredSaveError(null);

        const targetHost =
            selectedHost ||
            hosts.find(
                (h) =>
                    selectedHostIp &&
                    (h.ipAddress === selectedHostIp ||
                        h.primaryIP === selectedHostIp ||
                        h.name === selectedHostIp)
            ) ||
            hosts.find((h) => h.isCurrentHost) ||
            null;

        const resolvedHostId = targetHost?.hostId || targetHost?.id || currentHostId || "h-current";
        const resolvedHostName =
            targetHost?.name || targetHost?.hostname || targetHost?.ipAddress || targetHost?.primaryIP || "Current Host";
        const resolvedHostIp =
            targetHost?.ipAddress || targetHost?.primaryIP || selectedHostIp || undefined;

        let createdCred: KnowledgeCredentialItem = {
            id: `cred-inline-${Date.now()}`,
            principal: cleanPrincipal,
            secret: newCredSecret,
            kind: newCredKind,
            createdAt: new Date().toISOString(),
            hostId: resolvedHostId,
            hostName: resolvedHostName,
            hostPrimaryIP: resolvedHostIp,
            isCurrentHost: Boolean(targetHost?.isCurrentHost ?? !targetHost),
        };

        try {
            if (apolloClient) {
                const input: Record<string, any> = {
                    principal: cleanPrincipal,
                    secret: newCredSecret,
                    kind: newCredKind,
                };
                if (targetHost) {
                    if (
                        targetHost.networkHostId &&
                        !targetHost.networkHostId.startsWith("nh-custom-")
                    ) {
                        input.networkHostIDs = [targetHost.networkHostId];
                    }
                    if (targetHost.hostId && !targetHost.hostId.startsWith("nh-")) {
                        input.hostID = targetHost.hostId;
                    }
                } else if (currentHostId) {
                    input.hostID = currentHostId;
                }
                const res = await apolloClient.mutate({
                    mutation: CREATE_CREDENTIAL_MUTATION,
                    variables: { input },
                });
                const node = res.data?.createCredential;
                if (node?.id) {
                    createdCred = {
                        ...createdCred,
                        id: String(node.id),
                        principal: node.principal || createdCred.principal,
                        kind: node.kind || createdCred.kind,
                        secret: node.secret || createdCred.secret,
                        createdAt: node.createdAt || createdCred.createdAt,
                    };
                }
            }

            const nextCombined = [...allCombinedCreds, createdCred];
            setLocalCredentials((prev) => [...prev, createdCred]);
            setSelectedCredId(createdCred.id);
            const ip = selectedHostIp || createdCred.hostPrimaryIP || "";
            if (ip) {
                if (!selectedHostIp && createdCred.hostPrimaryIP) {
                    setSelectedHostIp(createdCred.hostPrimaryIP);
                }
                applySelection(createdCred.id, ip, nextCombined);
            } else {
                setUsername(createdCred.principal);
            }

            onCredentialCreated?.(createdCred, targetHost?.id);
            setIsAddingCred(false);
            setNewCredPrincipal("");
            setNewCredSecret("");
            setNewCredKind("KIND_PASSWORD");
        } catch (err: any) {
            setCredSaveError(err?.message || "Failed to create credential.");
        } finally {
            setIsSavingCred(false);
        }
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        const finalTarget = target.trim();
        if (finalTarget) {
            if (selectedCredId) {
                onConnect(finalTarget, selectedCredId, pivotType);
            } else {
                onConnect(finalTarget, undefined, pivotType);
            }
            handleClose();
        }
    };

    const handleStageSshExec = () => {
        if (!onLoadIntoComposer) return;
        const cred = allCombinedCreds.find((c) => c.id === selectedCredId);
        const { host: parsedHost, port: parsedPort } = splitHostAndPort(hostPort);
        const ip = selectedHostIp || cred?.hostPrimaryIP || parsedHost || "127.0.0.1";
        const portNum = parseInt(parsedPort, 10) || 22;
        const principal = username || cred?.principal || "root";
        const authArg = isSshKeyCredential(cred)
            ? `key=${JSON.stringify(cred?.secret || "")}`
            : `password=${JSON.stringify(password || cred?.secret || "")}`;
        const snippet = `res = ssh.exec(\n    "${ip}",\n    "${principal}",\n    "id && hostname",\n    ${authArg},\n    port=${portNum}\n)\nprint(res)`;
        onLoadIntoComposer(snippet);
        handleClose();
    };

    const handleClose = () => {
        setPivotType("SSH");
        setTarget("");
        setUsername("");
        setPassword("");
        setHostPort("");
        setSelectedCredId("");
        setSelectedHostId("");
        setSelectedHostIp("");
        setIsAddingCred(false);
        setCredSaveError(null);
        onClose();
    };

    return (
        <Transition show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={handleClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-200"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-150"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black/60" aria-hidden="true" />
                </Transition.Child>

                <div className="fixed inset-0 flex items-center justify-center p-4">
                    <Transition.Child
                        as={Fragment}
                        enter="ease-out duration-200"
                        enterFrom="opacity-0 scale-95"
                        enterTo="opacity-100 scale-100"
                        leave="ease-in duration-150"
                        leaveFrom="opacity-100 scale-100"
                        leaveTo="opacity-0 scale-95"
                    >
                        <Dialog.Panel className="w-full max-w-lg rounded-lg bg-gray-900 border border-gray-700 shadow-2xl">
                            <div className="flex items-center justify-between px-4 py-3 border-b border-gray-800">
                                <Dialog.Title className="text-white font-semibold text-sm flex items-center gap-2">
                                    <CornerDownRight className="w-4 h-4 text-purple-400" />
                                    Pivot
                                </Dialog.Title>
                                <button
                                    onClick={handleClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-4 w-4" />
                                </button>
                            </div>

                            <form onSubmit={handleSubmit} className="px-5 py-4 space-y-4">
                                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                    <div>
                                        <label htmlFor="pivot-type" className="block text-xs font-medium text-gray-300 mb-1.5">
                                            Pivot Type
                                        </label>
                                        <select
                                            id="pivot-type"
                                            data-testid="pivot-type-select"
                                            value={pivotType}
                                            onChange={(e) => setPivotType(e.target.value as PivotType)}
                                            className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm text-white focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        >
                                            {PIVOT_TYPE_OPTIONS.map((opt) => (
                                                <option key={opt.value} value={opt.value}>
                                                    {opt.label}
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                    <div className="sm:col-span-2">
                                        <label htmlFor="ssh-target" className="block text-xs font-medium text-gray-300 mb-1.5">
                                            Target
                                        </label>
                                        <input
                                            id="ssh-target"
                                            type="text"
                                            value={target}
                                            onChange={(e) => handleTargetChange(e.target.value)}
                                            placeholder="user@host:22 or host:22"
                                            className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                            autoFocus
                                        />
                                    </div>
                                </div>

                                <div className="rounded-md bg-[#151821] border border-[#2a3040] p-3 space-y-3">
                                    <div className="text-xs font-semibold text-purple-300 flex items-center justify-between">
                                        <span>Quick Fill</span>
                                        {selectedHost && (
                                            <button
                                                type="button"
                                                onClick={() => setSelectedHostId("")}
                                                className="text-[11px] font-normal text-gray-400 hover:text-gray-200 transition-colors"
                                            >
                                                Clear host filter
                                            </button>
                                        )}
                                    </div>

                                    <AvailablePivotsPicker
                                        hosts={hosts}
                                        selectedHostId={selectedHostId}
                                        onSelectHost={handleSelectHost}
                                    />

                                    <div className="space-y-1.5">
                                        <div className="flex items-center justify-between text-[11px] text-gray-400">
                                            <span className="flex items-center gap-1">
                                                <KeyRound size={11} className="text-amber-400" />
                                                <span>Credentials ({relevantCreds.length})</span>
                                                {selectedHost && (
                                                    <span className="text-blue-300">
                                                        · {selectedHost.name}
                                                    </span>
                                                )}
                                            </span>
                                            <button
                                                type="button"
                                                data-testid="ssh-add-credential-button"
                                                aria-label="Add credential"
                                                title="Add credential"
                                                onClick={() => {
                                                    setIsAddingCred((prev) => !prev);
                                                    setCredSaveError(null);
                                                }}
                                                className="p-0.5 rounded text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                                            >
                                                <Plus size={12} />
                                            </button>
                                        </div>

                                        {isAddingCred && (
                                            <InlineCredentialEditor
                                                principal={newCredPrincipal}
                                                kind={newCredKind}
                                                secret={newCredSecret}
                                                isSaving={isSavingCred}
                                                error={credSaveError}
                                                onPrincipalChange={setNewCredPrincipal}
                                                onKindChange={setNewCredKind}
                                                onSecretChange={setNewCredSecret}
                                                onSave={handleSaveInlineCredential}
                                                onCancel={() => {
                                                    setIsAddingCred(false);
                                                    setCredSaveError(null);
                                                }}
                                            />
                                        )}

                                        <CredentialPickerList
                                            credentials={relevantCreds}
                                            selectedCredId={selectedCredId}
                                            onSelectCred={handleSelectCred}
                                            filteredHostName={selectedHost?.name}
                                        />
                                    </div>
                                </div>

                                <div className="flex items-center justify-between pt-1">
                                    <div>
                                        {onLoadIntoComposer && pivotType === "SSH" && (
                                            <button
                                                type="button"
                                                onClick={handleStageSshExec}
                                                className="inline-flex items-center gap-1.5 text-xs font-medium py-1.5 px-2.5 rounded-md bg-[#252b3b] hover:bg-[#2f374b] text-purple-300 border border-[#363f54] transition-colors"
                                            >
                                                <Code2 size={13} />
                                                Stage in Eldritch Composer
                                            </button>
                                        )}
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <button
                                            type="button"
                                            onClick={handleClose}
                                            className="text-sm font-semibold py-1.5 px-3 rounded-md text-gray-300 hover:bg-[#2c3242] transition-colors"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            type="submit"
                                            disabled={!target.trim()}
                                            className="text-sm font-semibold py-1.5 px-3.5 rounded-md bg-purple-700 text-white hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                        >
                                            Connect
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </Dialog.Panel>
                    </Transition.Child>
                </div>
            </Dialog>
        </Transition>
    );
};

export const PivotModal = SshConnectionModal;
export default SshConnectionModal;
