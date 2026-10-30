import React, { Fragment, useContext, useEffect, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import { getApolloContext } from "@apollo/client";
import { KeyRound } from "lucide-react";
import { CREATE_CREDENTIAL_MUTATION } from "../../graphql";
import {
    KnowledgeCredentialItem,
    KnowledgeHostItem,
} from "../../hooks/useShellData";

export interface CredentialModalProps {
    isOpen: boolean;
    onClose: () => void;
    currentHostId?: string;
    currentHostName?: string;
    currentHostIp?: string;
    hosts?: KnowledgeHostItem[];
    onCredentialCreated?: (
        credential: KnowledgeCredentialItem,
        targetHostId?: string
    ) => void;
}

const CREDENTIAL_KIND_OPTIONS = [
    { value: "KIND_PASSWORD", label: "Password" },
    { value: "KIND_SSH_KEY", label: "SSH Key" },
    { value: "KIND_NTLM_HASH", label: "NTLM Hash" },
];

export const CredentialModal: React.FC<CredentialModalProps> = ({
    isOpen,
    onClose,
    currentHostId,
    currentHostName,
    currentHostIp,
    hosts = [],
    onCredentialCreated,
}) => {
    const apolloContext = useContext(getApolloContext());
    const apolloClient = apolloContext?.client;

    const [principal, setPrincipal] = useState("");
    const [secret, setSecret] = useState("");
    const [kind, setKind] = useState("KIND_PASSWORD");
    const [selectedHostKey, setSelectedHostKey] = useState("");
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [submitError, setSubmitError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) {
            return;
        }
        setPrincipal("");
        setSecret("");
        setKind("KIND_PASSWORD");
        setSelectedHostKey("");
        setIsSubmitting(false);
        setSubmitError(null);
    }, [isOpen]);

    const adjacentHosts = hosts.filter((h) => !h.isCurrentHost);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const cleanPrincipal = principal.trim();
        if (!cleanPrincipal || !secret.trim() || isSubmitting) {
            return;
        }

        setIsSubmitting(true);
        setSubmitError(null);

        const matchedHost = selectedHostKey
            ? adjacentHosts.find(
                  (h) =>
                      h.id === selectedHostKey ||
                      (h.networkHostId && `nh-${h.networkHostId}` === selectedHostKey) ||
                      (h.networkHostId && h.networkHostId === selectedHostKey) ||
                      (h.hostId && h.hostId === selectedHostKey)
              )
            : undefined;

        const resolvedHostId = matchedHost
            ? matchedHost.hostId || matchedHost.id
            : currentHostId || "h-current";
        const resolvedHostName = matchedHost
            ? matchedHost.name ||
              matchedHost.hostname ||
              matchedHost.ipAddress ||
              matchedHost.primaryIP ||
              matchedHost.id
            : currentHostName || "Current Host";
        const resolvedHostIp = matchedHost
            ? matchedHost.ipAddress || matchedHost.primaryIP
            : currentHostIp;
        const isCurrHost = !matchedHost;

        try {
            let createdCred: KnowledgeCredentialItem = {
                id: `cred-custom-${Date.now()}`,
                principal: cleanPrincipal,
                secret,
                kind,
                createdAt: new Date().toISOString(),
                hostId: resolvedHostId,
                hostName: resolvedHostName,
                hostPrimaryIP: resolvedHostIp,
                isCurrentHost: isCurrHost,
            };

            if (apolloClient) {
                try {
                    const input: Record<string, any> = {
                        principal: cleanPrincipal,
                        secret,
                        kind,
                    };
                    if (matchedHost) {
                        if (
                            matchedHost.networkHostId &&
                            !matchedHost.networkHostId.startsWith("nh-custom-")
                        ) {
                            input.networkHostIDs = [matchedHost.networkHostId];
                        }
                        if (
                            matchedHost.hostId &&
                            !matchedHost.hostId.startsWith("nh-")
                        ) {
                            input.hostID = matchedHost.hostId;
                        }
                    } else if (currentHostId) {
                        input.hostID = currentHostId;
                    }
                    const res = await apolloClient.mutate({
                        mutation: CREATE_CREDENTIAL_MUTATION,
                        variables: { input },
                    });
                    const node = res.data?.createCredential;
                    if (node) {
                        createdCred = {
                            ...createdCred,
                            id: String(node.id),
                            principal: node.principal || createdCred.principal,
                            kind: node.kind || createdCred.kind,
                            secret: node.secret || createdCred.secret,
                            createdAt: node.createdAt || createdCred.createdAt,
                        };
                    }
                } catch (err: any) {
                    const msg = String(err?.message || "");
                    if (!msg.includes("No more mocked responses")) {
                        throw err;
                    }
                }
            }

            onCredentialCreated?.(createdCred, matchedHost?.id);
            onClose();
        } catch (err: any) {
            setSubmitError(err?.message || "Failed to create credential.");
        } finally {
            setIsSubmitting(false);
        }
    };

    if (!isOpen) {
        return null;
    }

    const currentHostOptionLabel = `${currentHostName || "Current Host"}${
        currentHostIp ? ` (${currentHostIp})` : ""
    }`;

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
                            data-testid="credential-modal"
                            className="w-full max-w-md rounded-lg bg-[#1c202b] border border-[#343b4d] shadow-2xl"
                        >
                            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2e3444]">
                                <Dialog.Title className="text-white font-semibold text-sm flex items-center gap-2">
                                    <KeyRound size={15} className="text-gray-400" />
                                    <span>New Credential</span>
                                </Dialog.Title>
                                <button
                                    type="button"
                                    aria-label="Close credential modal"
                                    onClick={onClose}
                                    className="text-gray-400 hover:text-white transition-colors"
                                >
                                    <XMarkIcon className="h-4 w-4" />
                                </button>
                            </div>

                            <form onSubmit={handleSubmit} className="px-5 py-4 space-y-3.5">
                                <div>
                                    <label
                                        htmlFor="cred-principal"
                                        className="block text-xs font-medium text-gray-300 mb-1"
                                    >
                                        Principal / Username
                                    </label>
                                    <input
                                        id="cred-principal"
                                        type="text"
                                        value={principal}
                                        onChange={(e) => setPrincipal(e.target.value)}
                                        placeholder="root"
                                        autoFocus
                                        className="w-full px-3 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-gray-500"
                                    />
                                </div>

                                <div>
                                    <label
                                        htmlFor="cred-secret"
                                        className="block text-xs font-medium text-gray-300 mb-1"
                                    >
                                        Secret / Password
                                    </label>
                                    <input
                                        id="cred-secret"
                                        type="text"
                                        value={secret}
                                        onChange={(e) => setSecret(e.target.value)}
                                        placeholder="Password, SSH key, or hash"
                                        className="w-full px-3 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs font-mono text-white placeholder-gray-500 focus:outline-none focus:border-gray-500"
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-3">
                                    <div>
                                        <label
                                            htmlFor="cred-kind"
                                            className="block text-xs font-medium text-gray-300 mb-1"
                                        >
                                            Kind / Type
                                        </label>
                                        <select
                                            id="cred-kind"
                                            value={kind}
                                            onChange={(e) => setKind(e.target.value)}
                                            className="w-full px-2.5 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs text-white focus:outline-none focus:border-gray-500"
                                        >
                                            {CREDENTIAL_KIND_OPTIONS.map((opt) => (
                                                <option key={opt.value} value={opt.value}>
                                                    {opt.label}
                                                </option>
                                            ))}
                                        </select>
                                    </div>

                                    <div>
                                        <label
                                            htmlFor="cred-host"
                                            className="block text-xs font-medium text-gray-300 mb-1"
                                        >
                                            Host
                                        </label>
                                        <select
                                            id="cred-host"
                                            value={selectedHostKey}
                                            onChange={(e) => setSelectedHostKey(e.target.value)}
                                            className="w-full px-2.5 py-1.5 bg-[#12151c] border border-[#343b4d] rounded-md text-xs text-white focus:outline-none focus:border-gray-500"
                                        >
                                            <option value="">{currentHostOptionLabel}</option>
                                            {adjacentHosts.map((h) => {
                                                const ip = h.ipAddress || h.primaryIP;
                                                return (
                                                    <option key={h.id} value={h.id}>
                                                        {h.name}
                                                        {ip && ip !== h.name ? ` (${ip})` : ""}
                                                    </option>
                                                );
                                            })}
                                        </select>
                                    </div>
                                </div>

                                {submitError && (
                                    <div
                                        data-testid="credential-modal-error"
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
                                        data-testid="credential-modal-submit"
                                        disabled={!principal.trim() || !secret.trim() || isSubmitting}
                                        className="text-xs font-semibold py-1.5 px-3.5 rounded-md bg-gray-100 text-gray-900 hover:bg-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                                    >
                                        Create Credential
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

export default CredentialModal;

