import React from "react";
import { ArrowDownTrayIcon } from "@heroicons/react/24/outline";
import { KnowledgeFileItem } from "../../hooks/useShellData";
import { formatFileSize } from "../../utils/networkGraphUtils";

export interface GraphFilesSectionProps {
    files: KnowledgeFileItem[];
}

const FileRow: React.FC<{ file: KnowledgeFileItem }> = ({ file }) => {
    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="text-xs text-gray-200 font-mono truncate">{file.path}</div>
                <div className="text-[11px] text-gray-500 flex items-center gap-2 mt-0.5">
                    <span>{formatFileSize(file.size)}</span>
                    {file.hostName && <span>• {file.hostName}</span>}
                </div>
            </div>
            <a
                href={`/cdn/hostfiles/${file.id}`}
                download
                aria-label={`Download ${file.path}`}
                className="text-gray-400 hover:text-gray-200 p-1 rounded hover:bg-gray-800 shrink-0 transition-colors"
            >
                <ArrowDownTrayIcon className="w-4 h-4" />
            </a>
        </div>
    );
};

export const GraphFilesSection: React.FC<GraphFilesSectionProps> = ({ files }) => {
    return (
        <div className="px-4 py-3 border-b border-gray-800" data-testid="graph-files-section">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[11px] uppercase tracking-wider text-gray-400 font-medium">
                    Reported Files ({files.length})
                </span>
            </div>
            {files.length === 0 ? (
                <div className="py-2 text-xs text-gray-500">No files reported nearby.</div>
            ) : (
                <div className="divide-y divide-gray-800/60">
                    {files.map((f) => (
                        <FileRow key={f.id} file={f} />
                    ))}
                </div>
            )}
        </div>
    );
};

export default GraphFilesSection;

