import { Upload } from "lucide-react";
import { FC, useState } from "react";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";

type UploadAssetModalProps = {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    onUploadSuccess: () => void;
};

const UploadAssetModal: FC<UploadAssetModalProps> = ({ isOpen, setOpen, onUploadSuccess }) => {
    const [file, setFile] = useState<File | null>(null);
    const [fileName, setFileName] = useState<string>("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files[0]) {
            const selectedFile = e.target.files[0];
            setFile(selectedFile);
            setFileName(selectedFile.name);
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!file || !fileName) return;

        setLoading(true);
        setError(null);

        const formData = new FormData();
        formData.append("fileName", fileName);
        formData.append("fileContent", file);

        try {
            const response = await fetch("/cdn/upload", {
                method: "POST",
                body: formData,
            });

            if (!response.ok) {
                throw new Error("Upload failed");
            }

            onUploadSuccess();
            setOpen(false);
            setFile(null);
            setFileName("");
        } catch (err: any) {
            setError(err.message || "An error occurred during upload");
        } finally {
            setLoading(false);
        }
    };

    return (
        <Modal setOpen={setOpen} isOpen={isOpen} size="md">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">
                        Upload Asset
                    </h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Upload a new asset or update an existing one by name.
                    </p>
                </div>

                {error && (
                    <div className="p-4 bg-red-50 text-red-700 rounded-md border border-red-200">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="flex flex-col gap-2">
                        <label className="hidden" aria-hidden="true">
                            Select file or folder to upload
                        </label>
                        <input
                            type="file"
                            required
                            onChange={handleFileChange}
                            className="mt-1 block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-semibold file:bg-purple-50 file:text-purple-700 hover:file:bg-purple-100"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-700">
                            Asset Name
                        </label>
                        <input
                            type="text"
                            required
                            value={fileName}
                            onChange={(e) => setFileName(e.target.value)}
                            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                        />
                    </div>

                    <div className="flex justify-end gap-2 mt-4">
                        <Button
                            type="button"
                            onClick={() => setOpen(false)}
                            buttonVariant="outline"
                            buttonStyle={{ color: "gray", size: "md" }}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={loading || !file}
                            buttonVariant="solid"
                            buttonStyle={{ color: "purple", size: "md" }}
                            leftIcon={<Upload className="h-5 w-5" />}
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
