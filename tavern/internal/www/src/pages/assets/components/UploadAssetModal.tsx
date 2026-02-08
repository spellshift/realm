import { Upload, Folder, File as FileIcon, Loader2 } from "lucide-react";
import { FC, useRef } from "react";
import { useFormik } from "formik";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import { FileCard } from "./FileCard";
import { uploadFile } from "../utils";
import { FileItem } from "../types";

type UploadAssetModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    onUploadSuccess: () => void;
};

interface UploadFormValues {
    files: FileItem[];
}

const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB

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
                    await uploadFile(files[i].file, files[i].name, (progress) => {
                         files[i].progress = progress;
                         setFieldValue("files", [...files]);
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
