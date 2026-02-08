export type FileStatus = "pending" | "uploading" | "success" | "error";

export interface FileItem {
    id: string;
    file: File;
    name: string;
    status: FileStatus;
    progress: number;
    error?: string;
}
