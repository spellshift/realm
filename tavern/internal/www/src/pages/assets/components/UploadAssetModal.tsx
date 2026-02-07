import { Upload, X, CheckCircle, AlertCircle, File as FileIcon } from "lucide-react";
import { FC, useState } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import { formatBytes } from "../utils";

type UploadAssetModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    onUploadSuccess: () => void;
};

type FileStatus = 'pending' | 'uploading' | 'success' | 'error';

interface SelectedFile {
    file: File;
    status: FileStatus;
    error?: string;
}

const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB

const UploadAssetModal: FC<UploadAssetModalProps> = ({ isOpen, setOpen, onUploadSuccess }) => {
    const [selectedFiles, setSelectedFiles] = useState<SelectedFile[]>([]);
    const [uploading, setUploading] = useState(false);

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files) {
            const newFiles: SelectedFile[] = Array.from(e.target.files).map(file => ({
                file,
                status: 'pending'
            }));
            setSelectedFiles(prev => [...prev, ...newFiles]);
        }
        // Reset input value to allow selecting the same file again
        e.target.value = '';
    };

    const handleRemoveFile = (index: number) => {
        if (uploading) return;
        setSelectedFiles(prev => prev.filter((_, i) => i !== index));
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (selectedFiles.length === 0) return;

        setUploading(true);

        const filesToUpload = selectedFiles.map((f, i) => ({ file: f, index: i })).filter(item => item.file.status === 'pending' || item.file.status === 'error');

        for (const item of filesToUpload) {
            const { file, index } = item;

            // Check size
            if (file.file.size > MAX_FILE_SIZE) {
                 setSelectedFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[index] = { ...newFiles[index], status: 'error', error: `File exceeds 100MB limit` };
                    return newFiles;
                });
                continue;
            }

            setSelectedFiles(prev => {
                const newFiles = [...prev];
                newFiles[index] = { ...newFiles[index], status: 'uploading' };
                return newFiles;
            });

            const formData = new FormData();
            formData.append("fileName", file.file.name);
            formData.append("fileContent", file.file);

            try {
                const response = await fetch("/cdn/upload", {
                    method: "POST",
                    body: formData,
                });

                if (!response.ok) {
                    throw new Error("Upload failed");
                }

                setSelectedFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[index] = { ...newFiles[index], status: 'success', error: undefined };
                    return newFiles;
                });
            } catch (err: any) {
                setSelectedFiles(prev => {
                    const newFiles = [...prev];
                    newFiles[index] = { ...newFiles[index], status: 'error', error: err.message || "Upload failed" };
                    return newFiles;
                });
            }
        }

        setUploading(false);
        onUploadSuccess();
    };

    const handleClose = () => {
        if (!uploading) {
            setOpen(false);
            setSelectedFiles([]);
        }
    }

    return (
        <Modal setOpen={handleClose} isOpen={isOpen} size="lg">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Upload Assets
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Upload new assets. Max file size: 100MB.
                    </p>
                </div>

                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="flex flex-col gap-2">
                        <label className="hidden" aria-hidden="true">
                            Select files to upload
                        </label>
                        <input
                            type="file"
                            multiple
                            onChange={handleFileChange}
                            disabled={uploading}
                            className="mt-1 block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-semibold file:bg-purple-50 file:text-purple-700 hover:file:bg-purple-100 disabled:opacity-50 disabled:cursor-not-allowed"
                        />
                    </div>

                    {selectedFiles.length > 0 && (
                        <div className="flex flex-col gap-2 max-h-60 overflow-y-auto border border-gray-200 rounded-md p-2">
                            {selectedFiles.map((fileObj, index) => (
                                <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded text-sm">
                                    <div className="flex items-center gap-2 truncate flex-1">
                                        <FileIcon className="w-4 h-4 text-gray-400 flex-shrink-0" />
                                        <span className="truncate font-medium">{fileObj.file.name}</span>
                                        <span className="text-xs text-gray-500">({formatBytes(fileObj.file.size)})</span>
                                    </div>
                                    <div className="flex items-center gap-2">
                                        {fileObj.status === 'uploading' && <span className="text-xs text-blue-600">Uploading...</span>}
                                        {fileObj.status === 'success' && <CheckCircle className="w-4 h-4 text-green-500" />}
                                        {fileObj.status === 'error' && (
                                            <div className="flex items-center gap-1 text-red-500">
                                                <AlertCircle className="w-4 h-4" />
                                                <span className="text-xs">{fileObj.error}</span>
                                            </div>
                                        )}
                                        {fileObj.status === 'pending' && (
                                            <button
                                                type="button"
                                                onClick={() => handleRemoveFile(index)}
                                                disabled={uploading}
                                                className="text-gray-400 hover:text-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
                                            >
                                                <X className="w-4 h-4" />
                                            </button>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}

                    <div className="flex justify-end gap-2 mt-4">
                        <Button
                            type="button"
                            onClick={handleClose}
                            disabled={uploading}
                            buttonVariant="outline"
                            buttonStyle={{ color: "gray", size: "md" }}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={uploading || selectedFiles.length === 0 || selectedFiles.every(f => f.status === 'success')}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                            leftIcon={<Upload className="h-5 w-5" />}
                        >
                            {uploading ? "Uploading..." : "Upload"}
                        </Button>
                    </div>
                </form>
            </div>
        </Modal>
    );
};

export default UploadAssetModal;
