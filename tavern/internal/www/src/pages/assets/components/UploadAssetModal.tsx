import { Upload, Folder, File as FileIcon, X, CheckCircle, AlertCircle, Loader2 } from "lucide-react";
import { FC, useState, useRef } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";

type UploadAssetModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    onUploadSuccess: () => void;
};

type FileStatus = "pending" | "uploading" | "success" | "error";

interface FileWithStatus {
    file: File;
    status: FileStatus;
    errorMessage?: string;
}

const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB

const UploadAssetModal: FC<UploadAssetModalProps> = ({ isOpen, setOpen, onUploadSuccess }) => {
    const [files, setFiles] = useState<FileWithStatus[]>([]);
    const [singleFileName, setSingleFileName] = useState<string>("");
    const [loading, setLoading] = useState(false);
    const [uploadErrors, setUploadErrors] = useState<string[]>([]);
    const [progress, setProgress] = useState<{ current: number; total: number } | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);
    const folderInputRef = useRef<HTMLInputElement>(null);

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            const selectedFiles = Array.from(e.target.files).map(file => ({
                file,
                status: "pending" as FileStatus
            }));

            setFiles(prev => {
                const newFiles = [...prev, ...selectedFiles];
                // Update single file name logic based on the *new* combined list
                if (newFiles.length === 1) {
                    setSingleFileName(newFiles[0].file.name);
                } else {
                    setSingleFileName("");
                }
                return newFiles;
            });

            setUploadErrors([]);
            setProgress(null);
        }
    };

    const handleRemoveFile = (index: number) => {
        setFiles(prev => {
            const newFiles = prev.filter((_, i) => i !== index);
            if (newFiles.length === 1) {
                setSingleFileName(newFiles[0].file.name);
            } else {
                setSingleFileName("");
            }
            return newFiles;
        });
    };

    const handleClearAll = () => {
        setFiles([]);
        setSingleFileName("");
        setUploadErrors([]);
        setProgress(null);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (files.length === 0) return;

        setLoading(true);
        setUploadErrors([]);
        setProgress({ current: 0, total: files.length });

        const errors: string[] = [];

        for (let i = 0; i < files.length; i++) {
            const fileObj = files[i];

            // Skip already succeeded files if retrying?
            // For now assume we process all pending files or re-process everything if needed,
            // but usually this modal is fresh.

            if (fileObj.file.size > MAX_FILE_SIZE) {
                const errMsg = `File size exceeds 100MB limit`;
                errors.push(errMsg);
                setFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[i].status = "error";
                    newFiles[i].errorMessage = errMsg;
                    return newFiles;
                });
                setProgress({ current: i + 1, total: files.length });
                continue;
            }

            setFiles(prev => {
                const newFiles = [...prev];
                newFiles[i].status = "uploading";
                return newFiles;
            });

            let assetName = fileObj.file.webkitRelativePath || fileObj.file.name;

            if (files.length === 1 && singleFileName) {
                assetName = singleFileName;
            }

            const formData = new FormData();
            formData.append("fileName", assetName);
            formData.append("fileContent", fileObj.file);

            try {
                const response = await fetch("/cdn/upload", {
                    method: "POST",
                    body: formData,
                });

                if (!response.ok) {
                    throw new Error(`Failed to upload ${assetName}: ${response.statusText}`);
                }

                setFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[i].status = "success";
                    return newFiles;
                });

            } catch (err: any) {
                const errMsg = err.message || `Error uploading ${assetName}`;
                errors.push(errMsg);
                setFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[i].status = "error";
                    newFiles[i].errorMessage = errMsg;
                    return newFiles;
                });
            }

            setProgress({ current: i + 1, total: files.length });
        }

        setLoading(false);

        if (errors.length > 0) {
            setUploadErrors(errors);
            // If some succeeded, we might want to refresh the table anyway
            if (errors.length < files.length) {
                onUploadSuccess();
            }
        } else {
            onUploadSuccess();
            setOpen(false);
            setFiles([]);
            setSingleFileName("");
            setProgress(null);
        }
    };

    const handleClose = () => {
        setOpen(false);
        setFiles([]);
        setSingleFileName("");
        setUploadErrors([]);
        setProgress(null);
    }

    const getErrorDetails = () => {
        if (uploadErrors.length === 0) return "";
        const limit = 10;
        const visibleErrors = uploadErrors.slice(0, limit);
        let details = visibleErrors.join("\n");
        if (uploadErrors.length > limit) {
            details += `\n...and ${uploadErrors.length - limit} more errors`;
        }
        return details;
    };

    return (
        <Modal setOpen={handleClose} isOpen={isOpen} size="md">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Upload Assets
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Upload files or folders.
                    </p>
                </div>

                {uploadErrors.length > 0 && (
                    <div className="mb-4">
                        <AlertError
                            label="Upload completed with errors"
                            details={getErrorDetails()}
                        />
                        <button
                            type="button"
                            onClick={() => setUploadErrors([])}
                            className="mt-2 text-sm text-red-600 hover:text-red-500 underline"
                        >
                            Dismiss errors
                        </button>
                    </div>
                )}

                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="flex flex-col gap-2">
                        <label className="block text-sm font-medium text-gray-700">
                            Select Content
                        </label>
                        <div className="flex gap-3">
                            <input
                                type="file"
                                multiple
                                ref={fileInputRef}
                                onChange={handleFileChange}
                                className="hidden"
                                disabled={loading}
                            />
                            <input
                                type="file"
                                // @ts-ignore
                                webkitdirectory=""
                                mozdirectory=""
                                directory=""
                                ref={folderInputRef}
                                onChange={handleFileChange}
                                className="hidden"
                                disabled={loading}
                            />
                            <Button
                                type="button"
                                onClick={() => {
                                    if (fileInputRef.current) fileInputRef.current.value = "";
                                    fileInputRef.current?.click();
                                }}
                                disabled={loading}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<FileIcon className="h-4 w-4" />}
                            >
                                Add Files
                            </Button>
                            <Button
                                type="button"
                                onClick={() => {
                                    if (folderInputRef.current) folderInputRef.current.value = "";
                                    folderInputRef.current?.click();
                                }}
                                disabled={loading}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<Folder className="h-4 w-4" />}
                            >
                                Add Folder
                            </Button>
                             {files.length > 0 && !loading && (
                                <button
                                    type="button"
                                    onClick={handleClearAll}
                                    className="text-sm text-gray-500 hover:text-gray-700 underline ml-auto"
                                >
                                    Clear All
                                </button>
                            )}
                        </div>
                    </div>

                    {files.length === 1 && (
                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                Asset Name
                            </label>
                            <input
                                type="text"
                                required
                                value={singleFileName}
                                onChange={(e) => setSingleFileName(e.target.value)}
                                disabled={loading}
                                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            />
                        </div>
                    )}

                    {files.length > 1 && (
                        <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                Selected ({files.length} files)
                            </label>
                            <div className="max-h-96 overflow-y-auto border rounded-md bg-gray-50 text-sm text-gray-600 divide-y divide-gray-200">
                                {files.map((f, i) => (
                                    <div key={i} className="flex items-center justify-between p-2 hover:bg-gray-100">
                                        <div className="truncate flex-1 mr-2 flex items-center gap-2">
                                            {f.status === 'success' && <CheckCircle className="w-4 h-4 text-green-500" />}
                                            {f.status === 'error' && <AlertCircle className="w-4 h-4 text-red-500" />}
                                            {f.status === 'uploading' && <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />}
                                            {f.status === 'pending' && <div className="w-4 h-4" />} {/* Spacer */}
                                            <span className={f.status === 'error' ? 'text-red-500' : ''}>
                                                {f.file.webkitRelativePath || f.file.name}
                                            </span>
                                        </div>
                                        <button
                                            type="button"
                                            onClick={() => handleRemoveFile(i)}
                                            className={`text-gray-400 p-1 rounded-full ${loading ? 'opacity-50 cursor-not-allowed' : 'hover:text-red-500 hover:bg-gray-200'}`}
                                            title="Remove file"
                                            disabled={loading}
                                        >
                                            <X className="w-3 h-3" />
                                        </button>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {progress && (
                         <div>
                            <label className="block text-sm font-medium text-gray-700 mb-1">
                                Upload Progress
                            </label>
                            <div className="w-full bg-gray-200 rounded-full h-2.5">
                                <div
                                    className="bg-purple-600 h-2.5 rounded-full transition-all duration-300"
                                    style={{ width: `${(progress.current / progress.total) * 100}%` }}
                                ></div>
                            </div>
                            <p className="text-xs text-gray-500 mt-1 text-right">
                                {progress.current} / {progress.total}
                            </p>
                        </div>
                    )}

                    <div className="flex justify-end gap-2 mt-4">
                        <Button
                            type="button"
                            onClick={handleClose}
                            buttonVariant="outline"
                            buttonStyle={{ color: "gray", size: "md" }}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={loading || files.length === 0}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                            leftIcon={loading ? <Loader2 className="h-5 w-5 animate-spin" /> : <Upload className="h-5 w-5" />}
                        >
                            {loading ? "Uploading..." : "Upload"}
                        </Button>
                    </div>
                </form>
            </div>
        </Modal>
    );
};

export default UploadAssetModal;
