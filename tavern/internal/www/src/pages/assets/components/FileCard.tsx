import { Pencil, AlertTriangle, CheckCircle, Loader2, X } from "lucide-react";
import { useState } from "react";
import { FileItem } from "../types";
import { formatBytes } from "../../../utils/formatters";

interface FileCardProps {
    item: FileItem;
    index: number;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    formik: any;
    onRemove: () => void;
}

export const FileCard = ({ item, index, formik, onRemove }: FileCardProps) => {
    const [editingName, setEditingName] = useState<string | null>(null);

    const isPending = item.status === "pending";
    const isError = item.status === "error";
    const isSuccess = item.status === "success";
    const isUploading = item.status === "uploading";

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            saveEditing();
        } else if (e.key === 'Escape') {
            setEditingName(null);
        }
    };

    const saveEditing = () => {
        if (editingName && editingName.trim() !== "") {
            formik.setFieldValue(`files[${index}].name`, editingName.trim());
        }
        setEditingName(null);
    };

    return (
        <div className={`border rounded-md p-3 flex flex-col gap-2 ${
            isSuccess ? 'border-green-500 bg-green-50' :
            isError ? 'border-red-500 bg-red-50' :
            'bg-white border-gray-300'
        }`}>
            <div className="flex justify-between items-center gap-2">
                <div className="flex-1 min-w-0">
                    {editingName !== null ? (
                        <input
                            type="text"
                            autoFocus
                            value={editingName}
                            onChange={(e) => setEditingName(e.target.value)}
                            onBlur={saveEditing}
                            onKeyDown={handleKeyDown}
                            className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-1 border"
                        />
                    ) : (
                        <div className="flex items-center gap-2">
                            <span className="font-medium truncate text-gray-900" title={item.name}>
                                {item.name}
                            </span>
                            {isPending && (
                                <button
                                    type="button"
                                    onClick={() => setEditingName(item.name)}
                                    className="text-gray-400 hover:text-gray-600 focus:outline-none"
                                    title="Edit name"
                                >
                                    <Pencil className="w-3.5 h-3.5" />
                                </button>
                            )}
                                        {isError && (
                <div className="flex items-center gap-1 text-xs text-red-700 p-1 rounded">
                    <AlertTriangle className="w-4 h-4 shrink-0" />
                    <span>{item.error || "Upload failed"}</span>
                </div>
            )}
            {isUploading && (
                <div className="w-full bg-gray-200 rounded-full h-1.5 mt-1 overflow-hidden">
                    <div
                        className="bg-purple-600 h-1.5 rounded-full transition-all duration-300"
                        style={{ width: `${item.progress}%` }}
                    ></div>
                </div>
            )}
                        </div>
                    )}
                </div>

                <div className="flex items-center gap-3 shrink-0">
                    <span className="text-xs text-gray-500">{formatBytes(item.file.size)}</span>

                    {isSuccess && <CheckCircle className="w-5 h-5 text-green-500" />}
                    {isUploading && (
                        <div className="w-5 h-5 flex items-center justify-center">
                            <Loader2 className="w-4 h-4 text-purple-600 animate-spin" />
                        </div>
                    )}

                    {!isSuccess && !isUploading && (<button
                        type="button"
                        onClick={onRemove}
                        className={`text-gray-400 p-1 rounded-full hover:bg-gray-200 hover:text-gray-600 ${isUploading ? 'invisible' : ''}`}
                        title="Remove"
                    >
                        <X className="w-4 h-4" />
                    </button>
                    )}
                </div>
            </div>
        </div>
    );
};
