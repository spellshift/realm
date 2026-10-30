import React, { Fragment, useEffect, useMemo, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import { Code2, KeyRound, Server, Terminal } from "lucide-react";
import { KnowledgeCredentialItem, KnowledgeHostItem } from "../hooks/useShellData";

interface SshConnectionModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConnect: (target: string) => void;
    initialTarget?: string;
    credentials?: KnowledgeCredentialItem[];
    hosts?: KnowledgeHostItem[];
    onLoadIntoComposer?: (code: string) => void;
}

const formatCredKind = (kind?: string) => {
    if (!kind) return "CRED";
    return kind.replace(/^KIND_/, "");
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

const SshConnectionModal: React.FC<SshConnectionModalProps> = ({
    isOpen,
    onClose,
    onConnect,
    initialTarget = "",
    credentials = [],
    hosts = [],
    onLoadIntoComposer,
}) => {
    const [target, setTarget] = useState(initialTarget);
    const [username, setUsername] = useState<string>("");
    const [password, setPassword] = useState<string>("");
    const [hostPort, setHostPort] = useState<string>("");
    const [selectedHostIp, setSelectedHostIp] = useState<string>("");
    const [selectedPort, setSelectedPort] = useState<string>("22");
    const [selectedCredId, setSelectedCredId] = useState<string>("");

    useEffect(() => {
        if (isOpen) {
            const parsed = parseSshTarget(initialTarget);
            setTarget(initialTarget);
            setUsername(parsed.username);
            setPassword(parsed.password);
            setHostPort(parsed.hostPort);
            if (parsed.hostPort) {
                const lastColon = parsed.hostPort.lastIndexOf(":");
                if (lastColon > 0) {
                    setSelectedHostIp(parsed.hostPort.slice(0, lastColon));
                    setSelectedPort(parsed.hostPort.slice(lastColon + 1) || "22");
                } else {
                    setSelectedHostIp(parsed.hostPort);
                }
            }
        }
    }, [isOpen, initialTarget]);

    const relevantCreds = useMemo(() => {
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
        const activeHost = selectedHostIp || hostPort.split(":")[0] || "";
        return Array.from(byId.values()).sort((a, b) => {
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
    }, [credentials, hosts, selectedHostIp, hostPort]);

    const handleTargetChange = (nextTarget: string) => {
        setTarget(nextTarget);
        const parsed = parseSshTarget(nextTarget);
        setUsername(parsed.username);
        setPassword(parsed.password);
        setHostPort(parsed.hostPort);
    };

    const handleUsernameChange = (nextUsername: string) => {
        setUsername(nextUsername);
        const effectiveHostPort =
            hostPort || (selectedHostIp ? `${selectedHostIp}:${selectedPort || "22"}` : "");
        setTarget(buildSshTarget(nextUsername, password, effectiveHostPort));
    };

    const handlePasswordChange = (nextPassword: string) => {
        setPassword(nextPassword);
        const effectiveHostPort =
            hostPort || (selectedHostIp ? `${selectedHostIp}:${selectedPort || "22"}` : "");
        setTarget(buildSshTarget(username, nextPassword, effectiveHostPort));
    };

    const applySelection = (credId: string, hostIp: string, port: string) => {
        const cred = relevantCreds.find((c) => c.id === credId);
        const ip = hostIp || "127.0.0.1";
        const p = port || "22";
        const nextHostPort = `${ip}:${p}`;
        setHostPort(nextHostPort);
        if (cred) {
            setUsername(cred.principal);
            setPassword(cred.secret);
            setTarget(buildSshTarget(cred.principal, cred.secret, nextHostPort));
        } else if (hostIp) {
            setTarget(buildSshTarget(username || "root", password, nextHostPort));
        }
    };

    const handleSelectCred = (cred: KnowledgeCredentialItem) => {
        setSelectedCredId(cred.id);
        const ip = selectedHostIp || cred.hostPrimaryIP || "127.0.0.1";
        if (!selectedHostIp && cred.hostPrimaryIP) {
            setSelectedHostIp(cred.hostPrimaryIP);
        }
        applySelection(cred.id, ip, selectedPort);
    };

    const handleSelectHost = (host: KnowledgeHostItem) => {
        const ip = host.ipAddress || host.primaryIP || host.name;
        setSelectedHostIp(ip);
        applySelection(selectedCredId, ip, selectedPort);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        const finalTarget = target.trim();
        if (finalTarget) {
            onConnect(finalTarget);
            handleClose();
        }
    };

    const handleStageSshExec = () => {
        if (!onLoadIntoComposer) return;
        const cred = relevantCreds.find((c) => c.id === selectedCredId);
        const ip = selectedHostIp || cred?.hostPrimaryIP || hostPort.split(":")[0] || "127.0.0.1";
        const portNum = parseInt(selectedPort, 10) || 22;
        const principal = username || cred?.principal || "root";
        const secret = password || cred?.secret || "";
        const snippet = `res = ssh.exec(\n    "${ip}",\n    "${principal}",\n    "id && hostname",\n    password=${JSON.stringify(secret)},\n    port=${portNum}\n)\nprint(res)`;
        onLoadIntoComposer(snippet);
        handleClose();
    };

    const handleClose = () => {
        setTarget("");
        setUsername("");
        setPassword("");
        setHostPort("");
        setSelectedCredId("");
        setSelectedHostIp("");
        onClose();
    };

    const hasKnowledgeGraphSuggestions = relevantCreds.length > 0 || hosts.length > 0;

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
                                    <Terminal className="w-4 h-4 text-purple-400" />
                                    SSH Connection
                                </Dialog.Title>
                                <button
                                    onClick={handleClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-4 w-4" />
                                </button>
                            </div>

                            <form onSubmit={handleSubmit} className="px-5 py-4 space-y-4">
                                <div>
                                    <label htmlFor="ssh-target" className="block text-xs font-medium text-gray-300 mb-1.5">
                                        Target
                                    </label>
                                    <input
                                        id="ssh-target"
                                        type="text"
                                        value={target}
                                        onChange={(e) => handleTargetChange(e.target.value)}
                                        placeholder="host:port or user:password@host:port"
                                        className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        autoFocus
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-3">
                                    <div>
                                        <label htmlFor="ssh-username" className="block text-xs font-medium text-gray-300 mb-1.5">
                                            Username
                                        </label>
                                        <input
                                            id="ssh-username"
                                            type="text"
                                            value={username}
                                            onChange={(e) => handleUsernameChange(e.target.value)}
                                            placeholder="root"
                                            className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        />
                                    </div>
                                    <div>
                                        <label htmlFor="ssh-password" className="block text-xs font-medium text-gray-300 mb-1.5">
                                            Password
                                        </label>
                                        <input
                                            id="ssh-password"
                                            type="password"
                                            value={password}
                                            onChange={(e) => handlePasswordChange(e.target.value)}
                                            placeholder="••••••••"
                                            className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        />
                                    </div>
                                </div>

                                {hasKnowledgeGraphSuggestions && (
                                    <div className="rounded-md bg-[#151821] border border-[#2a3040] p-3 space-y-3">
                                        <div className="text-xs font-semibold text-purple-300 flex items-center justify-between">
                                            <span>Quick Fill</span>
                                            <div className="flex items-center gap-1.5 font-normal text-gray-400">
                                                <span>Port:</span>
                                                <input
                                                    type="text"
                                                    value={selectedPort}
                                                    onChange={(e) => {
                                                        setSelectedPort(e.target.value);
                                                        applySelection(selectedCredId, selectedHostIp, e.target.value);
                                                    }}
                                                    className="w-14 px-1.5 py-0.5 rounded bg-[#101218] border border-[#2e3444] text-xs font-mono text-gray-200"
                                                />
                                            </div>
                                        </div>

                                        {hosts.length > 0 && (
                                            <div className="space-y-1.5">
                                                <div className="text-[11px] text-gray-400 flex items-center gap-1">
                                                    <Server size={11} className="text-blue-400" />
                                                    <span>Known Hosts ({hosts.length})</span>
                                                </div>
                                                <div className="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto">
                                                    {hosts.map((h) => {
                                                        const ip = h.primaryIP || h.name;
                                                        const isSelected = selectedHostIp === ip;
                                                        return (
                                                            <button
                                                                key={h.id}
                                                                type="button"
                                                                onClick={() => handleSelectHost(h)}
                                                                className={`inline-flex items-center gap-1.5 px-2 py-1 rounded text-xs font-mono border transition-colors ${
                                                                    isSelected
                                                                        ? "bg-blue-600/30 border-blue-400 text-blue-100"
                                                                        : "bg-[#1d222e] hover:bg-[#262c3c] border-[#2f3647] text-gray-300"
                                                                }`}
                                                            >
                                                                <span>{h.name}</span>
                                                                {h.primaryIP && (
                                                                    <span className="text-gray-400">({h.primaryIP})</span>
                                                                )}
                                                                {h.isCurrentHost && (
                                                                    <span className="px-1 rounded bg-purple-500/20 text-purple-300 text-[10px]">
                                                                        this host
                                                                    </span>
                                                                )}
                                                            </button>
                                                        );
                                                    })}
                                                </div>
                                            </div>
                                        )}

                                        {relevantCreds.length > 0 && (
                                            <div className="space-y-1.5">
                                                <div className="text-[11px] text-gray-400 flex items-center gap-1">
                                                    <KeyRound size={11} className="text-amber-400" />
                                                    <span>Credentials ({relevantCreds.length})</span>
                                                </div>
                                                <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
                                                    {relevantCreds.map((c) => {
                                                        const isSelected = selectedCredId === c.id;
                                                        return (
                                                            <button
                                                                key={c.id}
                                                                type="button"
                                                                onClick={() => handleSelectCred(c)}
                                                                className={`flex items-center justify-between px-2.5 py-1.5 rounded text-xs border text-left transition-colors ${
                                                                    isSelected
                                                                        ? "bg-purple-600/30 border-purple-400 text-purple-100"
                                                                        : "bg-[#1d222e] hover:bg-[#262c3c] border-[#2f3647] text-gray-200"
                                                                }`}
                                                            >
                                                                <span className="font-mono font-medium truncate">
                                                                    {c.principal}
                                                                    <span className="text-gray-500 ml-1.5">
                                                                        ({ "*".repeat(Math.min(8, Math.max(4, c.secret?.length || 6))) })
                                                                    </span>
                                                                </span>
                                                                <span className="flex items-center gap-1.5 flex-shrink-0">
                                                                    <span className="px-1.5 py-0.5 rounded bg-[#12151c] text-gray-400 text-[10px] font-mono">
                                                                        {formatCredKind(c.kind)}
                                                                    </span>
                                                                    <span
                                                                        className={`px-1.5 py-0.5 rounded text-[10px] ${
                                                                            c.isCurrentHost
                                                                                ? "bg-emerald-500/20 text-emerald-300"
                                                                                : "bg-gray-700/60 text-gray-300"
                                                                        }`}
                                                                    >
                                                                        {c.hostName}
                                                                    </span>
                                                                </span>
                                                            </button>
                                                        );
                                                    })}
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                )}

                                <div className="flex items-center justify-between pt-1">
                                    <div>
                                        {onLoadIntoComposer && (
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

export default SshConnectionModal;
