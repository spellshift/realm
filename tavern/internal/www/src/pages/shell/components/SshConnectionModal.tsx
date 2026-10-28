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
    const [selectedHostIp, setSelectedHostIp] = useState<string>("");
    const [selectedPort, setSelectedPort] = useState<string>("22");
    const [selectedCredId, setSelectedCredId] = useState<string>("");

    useEffect(() => {
        if (isOpen && initialTarget) {
            setTarget(initialTarget);
        }
    }, [isOpen, initialTarget]);

    const passwordCreds = useMemo(
        () =>
            credentials.filter(
                (c) => !c.kind || c.kind === "KIND_PASSWORD" || c.kind === "PASSWORD"
            ),
        [credentials]
    );

    const applySelection = (credId: string, hostIp: string, port: string) => {
        const cred = credentials.find((c) => c.id === credId);
        const ip = hostIp || "127.0.0.1";
        const p = port || "22";
        if (cred) {
            setTarget(`${cred.principal}:${cred.secret}@${ip}:${p}`);
        } else if (hostIp) {
            const atIdx = target.lastIndexOf("@");
            const prefix = atIdx !== -1 ? target.slice(0, atIdx + 1) : "root@";
            setTarget(`${prefix}${ip}:${p}`);
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
        const ip = host.primaryIP || host.name;
        setSelectedHostIp(ip);
        applySelection(selectedCredId, ip, selectedPort);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (target.trim()) {
            onConnect(target.trim());
            setTarget("");
            setSelectedCredId("");
            setSelectedHostIp("");
            onClose();
        }
    };

    const handleStageSshExec = () => {
        if (!onLoadIntoComposer) return;
        const cred = credentials.find((c) => c.id === selectedCredId);
        const ip = selectedHostIp || cred?.hostPrimaryIP || "127.0.0.1";
        const portNum = parseInt(selectedPort, 10) || 22;
        const principal = cred?.principal || "root";
        const password = cred?.secret || "";
        const snippet = `res = ssh.exec(\n    "${ip}",\n    "${principal}",\n    "id && hostname",\n    password=${JSON.stringify(password)},\n    port=${portNum}\n)\nprint(res)`;
        onLoadIntoComposer(snippet);
        handleClose();
    };

    const handleClose = () => {
        setTarget("");
        setSelectedCredId("");
        setSelectedHostIp("");
        onClose();
    };

    const hasKnowledgeGraphSuggestions = passwordCreds.length > 0 || hosts.length > 0;

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
                        <Dialog.Panel className="w-full max-w-lg rounded-lg bg-[#1c202b] border border-[#343b4d] shadow-2xl">
                            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2e3444]">
                                <Dialog.Title className="text-white font-semibold text-base flex items-center gap-2">
                                    <Terminal size={17} className="text-purple-400" />
                                    SSH Connection
                                </Dialog.Title>
                                <button
                                    onClick={handleClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-5 w-5" />
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
                                        onChange={(e) => setTarget(e.target.value)}
                                        placeholder="user:password@host:port"
                                        className="w-full px-3 py-2 bg-[#12151c] border border-[#343b4d] rounded-md text-sm font-mono text-white placeholder-gray-500 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500"
                                        autoFocus
                                    />
                                    <p className="mt-1 text-xs text-gray-400">
                                        Negotiates SSH from the gRPC Tavern instance over the active portal stream.
                                    </p>
                                </div>

                                {hasKnowledgeGraphSuggestions && (
                                    <div className="rounded-md bg-[#151821] border border-[#2a3040] p-3 space-y-3">
                                        <div className="text-xs font-semibold text-purple-300 flex items-center justify-between">
                                            <span>Quick Fill from Knowledge Graph</span>
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

                                        {passwordCreds.length > 0 && (
                                            <div className="space-y-1.5">
                                                <div className="text-[11px] text-gray-400 flex items-center gap-1">
                                                    <KeyRound size={11} className="text-amber-400" />
                                                    <span>Captured Credentials ({passwordCreds.length})</span>
                                                </div>
                                                <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
                                                    {passwordCreds.map((c) => {
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
                                                                    <span className="px-1.5 py-0.2 rounded bg-[#12151c] text-gray-400 text-[10px] font-mono">
                                                                        {formatCredKind(c.kind)}
                                                                    </span>
                                                                    <span
                                                                        className={`px-1.5 py-0.2 rounded text-[10px] ${
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
