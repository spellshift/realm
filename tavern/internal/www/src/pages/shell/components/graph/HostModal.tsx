import React, { Fragment, useEffect, useMemo, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { BugAntIcon, XMarkIcon } from "@heroicons/react/24/outline";
import { useMutation } from "@apollo/client";
import { KeyRound } from "lucide-react";
import {
    CREATE_NETWORK_HOST_MUTATION,
    UPDATE_NETWORK_HOST_MUTATION,
} from "../../graphql";
import {
    KnowledgeCredentialItem,
    KnowledgeHostItem,
} from "../../hooks/useShellData";
import {
    formatPortListInput,
    formatSubnetCidr,
    parsePortList,
    togglePortInList,
} from "../../utils/networkGraphUtils";

export interface HostModalProps {
    isOpen: boolean;
    onClose: () => void;
    initialHost?: KnowledgeHostItem | null;
    availableCredentials?: KnowledgeCredentialItem[];
    onHostSaved?: (savedHost: KnowledgeHostItem, mode: "create" | "edit") => void;
}

const COMMON_QUICK_PORTS = [22, 80, 443, 445, 3389, 5432];

const formatCredKind = (kind?: string): string => {
    if (!kind) {
        return "PASSWORD";
    }
    return kind.replace(/^KIND_/, "");
};

interface HostPortPillsProps {
    ports: number[];
    openPortSet: Set<number>;
    onTogglePort: (port: number) => void;
}

const HostPortPills: React.FC<HostPortPillsProps> = ({
    ports,
    openPortSet,
    onTogglePort,
}) => {
    if (ports.length === 0) {
        return null;
    }

    return (
        <div
            className="flex items-center flex-wrap gap-1.5 mt-2"
            data-testid="host-port-pills"
        >
            {ports.map((port) => {
                const isOpen = openPortSet.has(port);
                const pillStyle = isOpen
                    ? "bg-gray-800 border-gray-600 text-gray-100"
                    : "bg-gray-900/50 border-gray-800 text-gray-500 line-through";
                const dotStyle = isOpen ? "bg-emerald-400" : "bg-gray-600";
                const actionLabel = isOpen ? `Close port ${port}` : `Open port ${port}`;

                return (
                    <button
                        key={port}
                        type="button"
                        onClick={() => onTogglePort(port)}
                        aria-pressed={isOpen}
                        aria-label={actionLabel}
                        data-testid={`host-port-pill-${port}`}
                        className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono border transition-colors hover:border-gray-500 ${pillStyle}`}
                    >
                        <span className={`w-1.5 h-1.5 rounded-full ${dotStyle}`} />
                        <span>:{port}</span>
                    </button>
                );
            })}
        </div>
    );
};

interface HostCredentialListProps {
    credentials: KnowledgeCredentialItem[];
    selectedIds: Set<string>;
    onToggleCredential: (credId: string) => void;
}

const HostCredentialList: React.FC<HostCredentialListProps> = ({
    credentials,
    selectedIds,
    onToggleCredential,
}) => {
    if (credentials.length === 0) {
        return (
            <div className="text-xs text-gray-500 py-1">
                No credentials available to associate yet.
            </div>
        );
    }

    return (
        <div
            className="flex flex-col gap-1 max-h-36 overflow-y-auto"
            data-testid="host-credential-selector"
        >
            {credentials.map((cred) => {
                const isSelected = selectedIds.has(cred.id);
                const rowClass = isSelected
                    ? "bg-gray-800 border-gray-500 text-white"
                    : "bg-[#12151c] hover:bg-[#191d27] border-[#2e3444] text-gray-300";

                return (
                    <button
                        key={cred.id}
                        type="button"
                        onClick={() => onToggleCredential(cred.id)}
                        aria-pressed={isSelected}
                        data-testid={`host-cred-option-${cred.id}`}
                        className={`flex items-center justify-between px-2.5 py-1.5 rounded text-xs border text-left transition-colors ${rowClass}`}
                    >
                        <span className="font-mono truncate">
                            <span className="font-medium">{cred.principal}</span>
                            {cred.hostName && (
                                <span className="text-gray-500 ml-1">@{cred.hostName}</span>
                            )}
                        </span>
                        <span className="flex items-center gap-1.5 shrink-0">
                            <span className="px-1.5 py-0.5 rounded bg-gray-900 text-gray-400 text-[10px] font-mono">
                                {formatCredKind(cred.kind)}
                            </span>
                            {isSelected && (
                                <span className="px-1.5 py-0.5 rounded bg-gray-700 text-gray-200 text-[10px]">
                                    Associated
                                </span>
                            )}
                        </span>
                    </button>
                );
            })}
        </div>
    );
};

export const HostModal: React.FC<HostModalProps> = ({
    isOpen,
    onClose,
    initialHost = null,
    availableCredentials = [],
    onHostSaved,
}) => {
    const [createNetworkHost] = useMutation(CREATE_NETWORK_HOST_MUTATION);
    const [updateNetworkHost] = useMutation(UPDATE_NETWORK_HOST_MUTATION);

    const isEditMode = Boolean(initialHost);

    const [ipAddress, setIpAddress] = useState("");
    const [name, setName] = useState("");
    const [portsInput, setPortsInput] = useState("");
    const [openPorts, setOpenPorts] = useState<number[]>([]);
    const [trackedPorts, setTrackedPorts] = useState<number[]>([]);
    const [selectedCredIds, setSelectedCredIds] = useState<Set<string>>(new Set());
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [submitError, setSubmitError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) {
            return;
        }
        const initIp = initialHost?.ipAddress || initialHost?.primaryIP || "";
        const initName = initialHost?.name || initialHost?.hostname || "";
        const initPorts = Array.isArray(initialHost?.openPorts)
            ? [...initialHost!.openPorts].sort((a, b) => a - b)
            : [];
        const initCredIds = new Set<string>(
            (initialHost?.credentials || []).map((c) => String(c.id))
        );

        setIpAddress(initIp);
        setName(initName === initIp ? "" : initName);
        setOpenPorts(initPorts);
        setPortsInput(formatPortListInput(initPorts));
        setTrackedPorts(initPorts.length > 0 ? initPorts : COMMON_QUICK_PORTS);
        setSelectedCredIds(initCredIds);
        setIsSubmitting(false);
        setSubmitError(null);
    }, [isOpen, initialHost]);

    const dedupedCredentials = useMemo(() => {
        const byId = new Map<string, KnowledgeCredentialItem>();
        for (const c of initialHost?.credentials || []) {
            if (c?.id) {
                byId.set(String(c.id), c);
            }
        }
        for (const c of availableCredentials) {
            if (c?.id && !byId.has(String(c.id))) {
                byId.set(String(c.id), c);
            }
        }
        return Array.from(byId.values());
    }, [initialHost, availableCredentials]);

    const openPortSet = useMemo(() => new Set(openPorts), [openPorts]);

    const displayedPills = useMemo(() => {
        const merged = new Set<number>(trackedPorts);
        for (const p of openPorts) {
            merged.add(p);
        }
        return Array.from(merged).sort((a, b) => a - b);
    }, [trackedPorts, openPorts]);

    const handlePortsInputChange = (nextRaw: string) => {
        setPortsInput(nextRaw);
        const parsed = parsePortList(nextRaw);
        setOpenPorts(parsed);
        setTrackedPorts((prev) => {
            const next = new Set(prev);
            for (const p of parsed) {
                next.add(p);
            }
            return Array.from(next).sort((a, b) => a - b);
        });
    };

    const handleTogglePort = (port: number) => {
        const nextOpen = togglePortInList(openPorts, port);
        setOpenPorts(nextOpen);
        setPortsInput(formatPortListInput(nextOpen));
        setTrackedPorts((prev) => {
            if (prev.includes(port)) {
                return prev;
            }
            return [...prev, port].sort((a, b) => a - b);
        });
    };

    const handleToggleCredential = (credId: string) => {
        setSelectedCredIds((prev) => {
            const next = new Set(prev);
            if (next.has(credId)) {
                next.delete(credId);
            } else {
                next.add(credId);
            }
            return next;
        });
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanIp = ipAddress.trim();
        if (!cleanIp || isSubmitting) {
            return;
        }

        setIsSubmitting(true);
        setSubmitError(null);

        const cleanName = name.trim() || initialHost?.name || cleanIp;
        const cleanHostname = name.trim() || initialHost?.hostname || "";
        const subnet = initialHost?.subnet || formatSubnetCidr(cleanIp);
        const credIdList = Array.from(selectedCredIds);

        const associatedCreds: KnowledgeCredentialItem[] = dedupedCredentials
            .filter((c) => selectedCredIds.has(c.id))
            .map((c) => ({
                ...c,
                hostName: cleanName,
                hostPrimaryIP: cleanIp,
            }));

        try {
            if (isEditMode && initialHost) {
                const initialCredIdSet = new Set(
                    (initialHost.credentials || []).map((c) => String(c.id))
                );
                const addCredentialIDs = credIdList.filter((id) => !initialCredIdSet.has(id));
                const removeCredentialIDs = Array.from(initialCredIdSet).filter(
                    (id) => !selectedCredIds.has(id)
                );

                let updatedHost: KnowledgeHostItem = {
                    ...initialHost,
                    name: cleanName,
                    hostname: cleanHostname,
                    ipAddress: cleanIp,
                    primaryIP: cleanIp,
                    subnet,
                    openPorts,
                    credentials: associatedCreds,
                    credentialCount: associatedCreds.length,
                };

                if (initialHost.networkHostId) {
                    const input: Record<string, any> = {
                        name: cleanName,
                        ipAddress: cleanIp,
                        openPorts,
                    };
                    if (cleanHostname) {
                        input.hostname = cleanHostname;
                    }
                    if (openPorts.length === 0) {
                        input.clearOpenPorts = true;
                    }
                    if (addCredentialIDs.length > 0) {
                        input.addCredentialIDs = addCredentialIDs;
                    }
                    if (removeCredentialIDs.length > 0) {
                        input.removeCredentialIDs = removeCredentialIDs;
                    }
                    const res = await updateNetworkHost({
                        variables: {
                            networkHostID: initialHost.networkHostId,
                            input,
                        },
                    });
                    const node = res.data?.updateNetworkHost;
                    if (node) {
                        updatedHost = {
                            ...updatedHost,
                            networkHostId: String(node.id),
                            name: node.name || updatedHost.name,
                            hostname: node.hostname ?? updatedHost.hostname,
                            ipAddress: node.ipAddress || updatedHost.ipAddress,
                            primaryIP: node.ipAddress || updatedHost.primaryIP,
                            subnet: node.subnet || updatedHost.subnet,
                            openPorts: Array.isArray(node.openPorts)
                                ? node.openPorts
                                : updatedHost.openPorts,
                        };
                    }
                } else {
                    const input: Record<string, any> = {
                        name: cleanName,
                        ipAddress: cleanIp,
                        openPorts,
                    };
                    if (cleanHostname) {
                        input.hostname = cleanHostname;
                    }
                    if (subnet) {
                        input.subnet = subnet;
                    }
                    if (initialHost.hostId) {
                        input.hostID = initialHost.hostId;
                    }
                    if (credIdList.length > 0) {
                        input.credentialIDs = credIdList;
                    }
                    const res = await createNetworkHost({
                        variables: { input },
                    });
                    const node = res.data?.createNetworkHost;
                    if (node) {
                        updatedHost = {
                            ...updatedHost,
                            networkHostId: String(node.id),
                        };
                    }
                }

                onHostSaved?.(updatedHost, "edit");
                onClose();
                return;
            }

            let createdHost: KnowledgeHostItem = {
                id: `nh-custom-${Date.now()}`,
                networkHostId: `nh-custom-${Date.now()}`,
                name: cleanName,
                hostname: cleanHostname,
                primaryIP: cleanIp,
                ipAddress: cleanIp,
                subnet,
                openPorts,
                tags: [],
                beaconCount: 0,
                credentialCount: associatedCreds.length,
                credentials: associatedCreds,
                isCurrentHost: false,
                sharedTags: [],
                sameSubnet: true,
                isReachable: true,
            };

            const input: Record<string, any> = {
                name: cleanName,
                ipAddress: cleanIp,
                openPorts,
            };
            if (cleanHostname) {
                input.hostname = cleanHostname;
            }
            if (subnet) {
                input.subnet = subnet;
            }
            if (credIdList.length > 0) {
                input.credentialIDs = credIdList;
            }
            const res = await createNetworkHost({
                variables: { input },
            });
            const node = res.data?.createNetworkHost;
            if (node) {
                createdHost = {
                    ...createdHost,
                    id: node.host?.id ? String(node.host.id) : `nh-${node.id}`,
                    hostId: node.host?.id ? String(node.host.id) : undefined,
                    networkHostId: String(node.id),
                    name: node.name || createdHost.name,
                    hostname: node.hostname || createdHost.hostname,
                    ipAddress: node.ipAddress || createdHost.ipAddress,
                    primaryIP: node.ipAddress || createdHost.primaryIP,
                    subnet: node.subnet || createdHost.subnet,
                    openPorts: Array.isArray(node.openPorts)
                        ? node.openPorts
                        : createdHost.openPorts,
                };
            }

            onHostSaved?.(createdHost, "create");
            onClose();
        } catch (err: any) {
            setSubmitError(err?.message || "Failed to save host.");
        } finally {
            setIsSubmitting(false);
        }
    };

    if (!isOpen) {
        return null;
    }

    const modalTitle = isEditMode ? "Edit Host" : "New Host";
    const submitLabel = isEditMode ? "Save Host" : "Add Host";

    return (
        <Transition show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={onClose}>
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
                        <Dialog.Panel
                            data-testid="host-modal"
                            className="w-full max-w-md rounded-lg bg-[#1c202b] border border-[#343b4d] shadow-2xl"
                        >
                            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2e3444]">
                                <Dialog.Title className="text-white font-semibold text-sm flex items-center gap-2">
                                    <BugAntIcon className="h-4 w-4 text-gray-400" />
                                    <span>{modalTitle}</span>
                                </Dialog.Title>
                                <button
                                    type="button"
                                    aria-label="Close host modal"
                                    onClick={onClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-4 w-4" />
                                </button>
                            </div>

                            <form onSubmit={handleSubmit} className="px-5 py-4 space-y-3.5">
                                <div className="grid grid-cols-2 gap-3">
                                    <div>
                                        <label
                                            htmlFor="host-ip-address"
                                            className="block text-xs font-medium text-gray-300 mb-1"
                                        >
                                            IP Address
                                        </label>
                                        <input
                                            id="host-ip-address"
                                            type="text"
                                            value={ipAddress}
                                            onChange={(e) => setIpAddress(e.target.value)}
                                            placeholder="10.0.0.25"
                                            autoFocus
                                            className="w-full px-3 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-gray-500"
                                        />
                                    </div>
                                    <div>
                                        <label
                                            htmlFor="host-name-input"
                                            className="block text-xs font-medium text-gray-300 mb-1"
                                        >
                                            Name / Hostname
                                        </label>
                                        <input
                                            id="host-name-input"
                                            type="text"
                                            value={name}
                                            onChange={(e) => setName(e.target.value)}
                                            placeholder="db-01.corp.local"
                                            className="w-full px-3 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-gray-500"
                                        />
                                    </div>
                                </div>

                                <div>
                                    <div className="flex items-center justify-between mb-1">
                                        <label
                                            htmlFor="host-open-ports"
                                            className="block text-xs font-medium text-gray-300"
                                        >
                                            Open Ports
                                        </label>
                                        <span className="text-[11px] text-gray-500">
                                            Click port to toggle open/closed
                                        </span>
                                    </div>
                                    <input
                                        id="host-open-ports"
                                        type="text"
                                        value={portsInput}
                                        onChange={(e) => handlePortsInputChange(e.target.value)}
                                        placeholder="22, 80, 443"
                                        className="w-full px-3 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-gray-500"
                                    />
                                    <HostPortPills
                                        ports={displayedPills}
                                        openPortSet={openPortSet}
                                        onTogglePort={handleTogglePort}
                                    />
                                </div>

                                <div>
                                    <div className="flex items-center justify-between mb-1.5">
                                        <span className="text-xs font-medium text-gray-300 flex items-center gap-1.5">
                                            <KeyRound size={12} className="text-gray-400" />
                                            <span>Credentials</span>
                                        </span>
                                        {selectedCredIds.size > 0 && (
                                            <span className="text-[11px] text-gray-400">
                                                {selectedCredIds.size} associated
                                            </span>
                                        )}
                                    </div>
                                    <HostCredentialList
                                        credentials={dedupedCredentials}
                                        selectedIds={selectedCredIds}
                                        onToggleCredential={handleToggleCredential}
                                    />
                                </div>

                                {submitError && (
                                    <div
                                        data-testid="host-modal-error"
                                        className="rounded bg-red-500/10 border border-red-500/30 px-3 py-2 text-xs text-red-300"
                                    >
                                        {submitError}
                                    </div>
                                )}

                                <div className="flex items-center justify-end gap-2 pt-1">
                                    <button
                                        type="button"
                                        onClick={onClose}
                                        className="text-xs font-medium py-1.5 px-3 rounded-md text-gray-300 hover:bg-[#2c3242] transition-colors"
                                    >
                                        Cancel
                                    </button>
                                    <button
                                        type="submit"
                                        data-testid="host-modal-submit"
                                        disabled={!ipAddress.trim() || isSubmitting}
                                        className="text-xs font-semibold py-1.5 px-3.5 rounded-md bg-gray-100 text-gray-900 hover:bg-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                                    >
                                        {submitLabel}
                                    </button>
                                </div>
                            </form>
                        </Dialog.Panel>
                    </Transition.Child>
                </div>
            </Dialog>
        </Transition>
    );
};

export default HostModal;

