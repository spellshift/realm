
export const formatBytes = (bytes: number, decimals = 0) => {
    if (!+bytes) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export const generateRandomString = (length: number) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    const randomValues = new Uint32Array(length);
    crypto.getRandomValues(randomValues);
    for (let i = 0; i < length; i++) {
        result += chars[randomValues[i] % chars.length];
    }
    return result;
};

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
