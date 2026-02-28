import { FileItem } from "./FileCard";

export const MAX_FILE_SIZE = 512 * 1024 * 1024; // 512MB
const CHUNK_SIZE = 10 * 1024 * 1024; // 10MB chunks to stay safely under GCP's 32MB limit

interface uploadFileParams {
    files: FileItem[];
    setFieldValue: (field: string, value: any) => void;
    onUploadSuccess: () => void;
}

export const uploadFiles = async ({files, setFieldValue, onUploadSuccess }: uploadFileParams) => {
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
            // Corrected your error message to match the 512MB limit
            files[i].error = "File size exceeds 512MB limit";
            setFieldValue("files", [...files]);
            hasErrors = true;
            continue;
        }

        try {
            const file = files[i].file;
            // Force at least 1 chunk just in case of an empty 0-byte file
            const totalChunks = Math.max(1, Math.ceil(file.size / CHUNK_SIZE));

            for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
                const start = chunkIndex * CHUNK_SIZE;
                const end = Math.min(start + CHUNK_SIZE, file.size);
                const chunk = file.slice(start, end);

                await new Promise<void>((resolve, reject) => {
                    const xhr = new XMLHttpRequest();
                    // You may want to point this to a specific chunking endpoint
                    xhr.open("POST", "/cdn/upload");

                    const formData = new FormData();
                    formData.append("fileName", files[i].name);
                    formData.append("chunkIndex", chunkIndex.toString());
                    formData.append("totalChunks", totalChunks.toString());
                    formData.append("fileContent", chunk);

                    xhr.upload.onprogress = (event) => {
                        if (event.lengthComputable) {
                            // Calculate overall progress across all chunks
                            const loadedOverall = start + event.loaded;
                            const percentComplete = (loadedOverall / file.size) * 100;

                            // Cap at 100 to avoid UI glitches
                            files[i].progress = Math.min(percentComplete, 100);
                            setFieldValue("files", [...files]);
                        }
                    };

                    xhr.onload = () => {
                        if (xhr.status >= 200 && xhr.status < 300) {
                            resolve();
                        } else {
                            reject(new Error(xhr.statusText || "Chunk upload failed"));
                        }
                    };

                    xhr.onerror = () => reject(new Error("Network error"));
                    xhr.send(formData);
                });
            }

            // Only mark as complete when the entire chunk loop finishes
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
};
