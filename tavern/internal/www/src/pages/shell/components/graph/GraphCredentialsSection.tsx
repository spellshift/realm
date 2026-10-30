import React, { useState } from "react";
import { ClipboardIcon, EyeIcon, EyeSlashIcon } from "@heroicons/react/24/outline";
import { Plus } from "lucide-react";
import {
    KnowledgeCredentialItem,
    KnowledgeHostItem,
} from "../../hooks/useShellData";
import CredentialModal from "./CredentialModal";

export interface GraphCredentialsSectionProps {
    credentials: KnowledgeCredentialItem[];
    currentHostId?: string;
    currentHostName?: string;
    currentHostIp?: string;
    hosts?: KnowledgeHostItem[];
    onOpenSshWithTarget: (target: string) => void;
    onCredentialCreated?: (
        credential: KnowledgeCredentialItem,
        targetHostId?: string
    ) => void;
}

const SecretToggleIcon: React.FC<{ isRevealed: boolean }> = ({ isRevealed }) => {
    if (isRevealed) {
        return <EyeSlashIcon className="w-4 h-4" />;
    }
    return <EyeIcon className="w-4 h-4" />;
};

const CredentialRow: React.FC<{
    cred: KnowledgeCredentialItem;
    currentHostIp?: string;
    onOpenSshWithTarget: (target: string) => void;
}> = ({ cred, currentHostIp, onOpenSshWithTarget }) => {
    const [isRevealed, setIsRevealed] = useState(false);
    const targetIp = cred.hostPrimaryIP || currentHostIp || "127.0.0.1";
    const sshTarget = `${cred.principal}:${cred.secret}@${targetIp}:22`;

    const handleCopy = () => {
        if (navigator.clipboard?.writeText) {
            navigator.clipboard.writeText(cred.secret).catch(() => {});
        }
    };

    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 text-xs text-gray-200 font-mono truncate">
                    <span className="font-medium">{cred.principal}</span>
                    <span className="text-gray-500">@{cred.hostName}</span>
                </div>
                <div className="flex items-center gap-1.5 mt-0.5 text-xs text-gray-400 font-mono">
                    <span className="truncate max-w-[180px]">
                        {isRevealed ? cred.secret : "••••••••"}
                    </span>
                    <button
                        type="button"
                        onClick={() => setIsRevealed((prev) => !prev)}
                        aria-label={isRevealed ? "Hide secret" : "Reveal secret"}
                        className="text-gray-400 hover:text-gray-200 p-0.5 transition-colors"
                    >
                        <SecretToggleIcon isRevealed={isRevealed} />
                    </button>
                    <button
                        type="button"
                        onClick={handleCopy}
                        aria-label="Copy secret"
                        className="text-gray-400 hover:text-gray-200 p-0.5 transition-colors"
                    >
                        <ClipboardIcon className="w-4 h-4" />
                    </button>
                </div>
            </div>
            <button
                type="button"
                onClick={() => onOpenSshWithTarget(sshTarget)}
                className="text-xs font-medium text-gray-200 hover:text-white px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 shrink-0 transition-colors"
            >
                SSH Pivot
            </button>
        </div>
    );
};

export const GraphCredentialsSection: React.FC<GraphCredentialsSectionProps> = ({
    credentials,
    currentHostId,
    currentHostName,
    currentHostIp,
    hosts = [],
    onOpenSshWithTarget,
    onCredentialCreated,
}) => {
    const [modalOpen, setModalOpen] = useState(false);

    return (
        <div className="px-3 py-3 border-b border-gray-800" data-testid="graph-credentials-section">
            <div className="flex items-center justify-between mb-1">
                <span className="text-xs uppercase tracking-wider text-gray-400 font-semibold">
                    Credentials ({credentials.length})
                </span>
                <button
                    type="button"
                    onClick={() => setModalOpen(true)}
                    aria-label="Add credential"
                    title="Create credential"
                    data-testid="add-credential-button"
                    className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                >
                    <Plus className="w-4 h-4" />
                </button>
            </div>
            {credentials.length === 0 ? (
                <div className="py-2 text-xs text-gray-500">No credentials reported nearby.</div>
            ) : (
                <div className="divide-y divide-gray-800/60">
                    {credentials.map((cred) => (
                        <CredentialRow
                            key={cred.id}
                            cred={cred}
                            currentHostIp={currentHostIp}
                            onOpenSshWithTarget={onOpenSshWithTarget}
                        />
                    ))}
                </div>
            )}

            {modalOpen && (
                <CredentialModal
                    isOpen={modalOpen}
                    onClose={() => setModalOpen(false)}
                    currentHostId={currentHostId}
                    currentHostName={currentHostName}
                    currentHostIp={currentHostIp}
                    hosts={hosts}
                    onCredentialCreated={onCredentialCreated}
                />
            )}
        </div>
    );
};

export default GraphCredentialsSection;
