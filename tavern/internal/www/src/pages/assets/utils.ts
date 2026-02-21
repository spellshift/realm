import { Steps, CreateToastFnReturn } from "@chakra-ui/react";
import { LinkEdge } from "../../utils/interfacesQuery";

const API_ENDPOINT = process.env.REACT_APP_API_ENDPOINT ?? 'http://localhost:8000';

export const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export const truncateAssetName = (name: string, maxLength: number = 25): string => {
    if (name.length <= maxLength) return name;

    // Check for path structure (forward or backward slashes)
    const hasPath = name.includes('/') || name.includes('\\');

    if (hasPath) {
        // Handle path truncation: prioritize keeping the filename
        const separator = name.includes('/') ? '/' : '\\';
        const parts = name.split(separator);
        const fileName = parts.pop() || "";

        // If filename itself is too long, truncate it
        if (fileName.length > maxLength) {
            return fileName.substring(0, maxLength - 3) + "...";
        }

        // Try to add parent directories until limit is reached
        let result = fileName;
        // Start from end of parts (deepest folder)
        for (let i = parts.length - 1; i >= 0; i--) {
            const part = parts[i];
            const potential = part + separator + result;
            // +3 for "..." prefix
            if (potential.length + 3 <= maxLength) {
                result = potential;
            } else {
                return "..." + separator + result;
            }
        }
        // Should not reach here if length check passed, but fallback
        return "..." + separator + result;
    }

    // Standard string truncation
    return name.substring(0, maxLength - 3) + "...";
};

export const generateRandomLinkPath = (length: number = 8) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    const randomValues = new Uint32Array(length);
    crypto.getRandomValues(randomValues);
    for (let i = 0; i < length; i++) {
        result += chars[randomValues[i] % chars.length];
    }
    return result;
};

export const handleCopyLink = (path: string, toast: CreateToastFnReturn) => {
    const url = `${API_ENDPOINT}/cdn/${path}`;
    navigator.clipboard.writeText(url);
    toast({
        title: "Link copied to clipboard",
        status: "success",
        duration: 2000,
        isClosable: true,
    });
};

export const sortLinks = (links: LinkEdge[]) => {
    return [...links].sort((a: LinkEdge, b: LinkEdge) => {
        const now = new Date();
        const aExpired = new Date(a.node.expiresAt) < now;
        const bExpired = new Date(b.node.expiresAt) < now;

        // 1. Unexpired first
        if (aExpired !== bExpired) {
            return aExpired ? 1 : -1;
        }

        // 2. Unlimited downloads first
        const aUnlimited = a.node.downloadLimit === null;
        const bUnlimited = b.node.downloadLimit === null;
        if (aUnlimited !== bUnlimited) {
            return aUnlimited ? -1 : 1;
        }

        // 3. Most remaining downloads first
        if (!aUnlimited && !bUnlimited) {
            const aRemaining = (a.node.downloadLimit || 0) - a.node.downloads;
            const bRemaining = (b.node.downloadLimit || 0) - b.node.downloads;
            return bRemaining - aRemaining;
        }

        return 0;
    });
}
