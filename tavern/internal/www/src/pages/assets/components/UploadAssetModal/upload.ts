import { FileItem } from "./FileCard";

export const MAX_FILE_SIZE = 512 * 1024 * 1024; // 512MB (see upload.go for server limit)

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
};
