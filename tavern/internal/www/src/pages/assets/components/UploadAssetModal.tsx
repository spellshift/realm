import { Upload, Folder, File as FileIcon, X, CheckCircle, Loader2, Pencil, AlertTriangle } from "lucide-react";
import { FC, useState, useRef } from "react";
import { useFormik } from "formik";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import * as Yup from "yup";

type UploadAssetModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    onUploadSuccess: () => void;
};

type FileStatus = "pending" | "uploading" | "success" | "error";

interface FileItem {
    id: string;
    file: File;
    name: string;
    status: FileStatus;
    progress: number;
    error?: string;
}

interface UploadFormValues {
    files: FileItem[];
}

const formatBytes = (bytes: number, decimals = 2) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB

interface FileCardProps {
    item: FileItem;
    index: number;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    formik: any;
    onRemove: () => void;
}

const FileCard = ({ item, index, formik, onRemove }: FileCardProps) => {
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

const UploadAssetModal: FC<UploadAssetModalProps> = ({ isOpen, setOpen, onUploadSuccess }) => {
    const fileInputRef = useRef<HTMLInputElement>(null);
    const folderInputRef = useRef<HTMLInputElement>(null);

    const formik = useFormik<UploadFormValues>({
        initialValues: {
            files: [],
        },
        onSubmit: async (values, { setFieldValue }) => {
            const files = [...values.files];
            let hasErrors = false;

            for (let i = 0; i < files.length; i++) {
                if (files[i].status === 'success') continue;

                // Update status to uploading
                files[i].status = 'uploading';
                files[i].progress = 0;
                files[i].error = undefined;
                setFieldValue("files", [...files]);

                if (files[i].file.size > MAX_FILE_SIZE) {
                    files[i].status = 'error';
                    files[i].error = "File size exceeds 100MB limit";
                    setFieldValue("files", [...files]);
                    hasErrors = true;
                    continue;
                }

                try {
                    await new Promise<void>((resolve, reject) => {
                        const xhr = new XMLHttpRequest();
                        xhr.open("POST", "/cdn/upload");

                        const formData = new FormData();
                        formData.append("fileName", files[i].name);
                        formData.append("fileContent", files[i].file);

                        xhr.upload.onprogress = (event) => {
                            if (event.lengthComputable) {
                                const percentComplete = (event.loaded / event.total) * 100;
                                files[i].progress = percentComplete;
                                // Force update. Note: This might be performance heavy if many files upload at once,
                                // but we are doing sequential uploads here (await in loop).
                                setFieldValue("files", [...files]);
                            }
                        };

                        xhr.onload = () => {
                            if (xhr.status >= 200 && xhr.status < 300) {
                                resolve();
                            } else {
                                reject(new Error(xhr.statusText || "Upload failed"));
                            }
                        };

                        xhr.onerror = () => reject(new Error("Network error"));
                        xhr.send(formData);
                    });

                    files[i].status = 'success';
                    files[i].progress = 100;
                    setFieldValue("files", [...files]);

                } catch (err: any) {
                    files[i].status = 'error';
                    files[i].error = err.message || "Upload failed";
                    setFieldValue("files", [...files]);
                    hasErrors = true;
                }
            }

            if (!hasErrors) {
                onUploadSuccess();
                // We keep the modal open to show success state, user can close it.
            }
        },
    });

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const newFiles: FileItem[] = Array.from(e.target.files).map((file: any) => ({
                id: Math.random().toString(36).substring(7) + Date.now(),
                file,
                name: file.webkitRelativePath || file.name,
                status: "pending",
                progress: 0
            }));
            formik.setFieldValue("files", [...formik.values.files, ...newFiles]);

            // Reset inputs
            if (fileInputRef.current) fileInputRef.current.value = "";
            if (folderInputRef.current) folderInputRef.current.value = "";
        }
    };

    const handleRemoveFile = (index: number) => {
        const newFiles = [...formik.values.files];
        newFiles.splice(index, 1);
        formik.setFieldValue("files", newFiles);
    };

    const handleClearAll = () => {
        formik.setFieldValue("files", []);
    };

    const isSubmitting = formik.isSubmitting;

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="lg">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Upload Assets
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Upload files or folders.
                    </p>
                </div>

                <form onSubmit={formik.handleSubmit} className="flex flex-col gap-4">
                    <div className="flex flex-col gap-2">
                        <div className="flex justify-between items-center">
                            <label className="block text-sm font-medium text-gray-700">
                                Select Content
                            </label>
                            {formik.values.files.length > 0 && !isSubmitting && (
                                <button
                                    type="button"
                                    onClick={handleClearAll}
                                    className="text-sm text-gray-500 hover:text-gray-700 underline"
                                >
                                    Clear All
                                </button>
                            )}
                        </div>

                        <div className="flex gap-3">
                            <input
                                type="file"
                                multiple
                                ref={fileInputRef}
                                onChange={handleFileChange}
                                className="hidden"
                                disabled={isSubmitting}
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
                                disabled={isSubmitting}
                            />
                            <Button
                                type="button"
                                onClick={() => fileInputRef.current?.click()}
                                disabled={isSubmitting}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<FileIcon className="h-4 w-4" />}
                            >
                                Add Files
                            </Button>
                            <Button
                                type="button"
                                onClick={() => folderInputRef.current?.click()}
                                disabled={isSubmitting}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                leftIcon={<Folder className="h-4 w-4" />}
                            >
                                Add Folder
                            </Button>
                        </div>
                    </div>

                    {formik.values.files.length > 0 && (
                        <div className="flex flex-col gap-3 max-h-[400px] overflow-y-auto pr-1">
                            {formik.values.files.map((item, index) => (
                                <FileCard
                                    key={item.id}
                                    item={item}
                                    index={index}
                                    formik={formik}
                                    onRemove={() => handleRemoveFile(index)}
                                />
                            ))}
                        </div>
                    )}

                    <div className="flex justify-end gap-2 mt-4 pt-4 border-t border-gray-100">
                        <Button
                            type="button"
                            onClick={() => setOpen(false)}
                            buttonVariant="outline"
                            buttonStyle={{ color: "gray", size: "md" }}
                            disabled={isSubmitting}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={isSubmitting || formik.values.files.length === 0}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                            leftIcon={isSubmitting ? <Loader2 className="h-5 w-5 animate-spin" /> : <Upload className="h-5 w-5" />}
                        >
                            {isSubmitting ? "Uploading..." : "Upload"}
                        </Button>
                    </div>
                </form>
            </div>
        </Modal>
    );
};

export default UploadAssetModal;
